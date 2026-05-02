use ai_ledger_core::{PrivacyMode, RunId};
use ai_ledger_event::{
    DatasetManifestCreatedPayload, EvalCaseFailedPayload, EventEnvelope, EventPayload, GateFailure,
    LlmRequestPayload, ReleaseGateFailedPayload,
};
use ai_ledger_log::JsonlEventLog;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "ai-ledger")]
#[command(about = "Privacy-first AI observability and assurance ledger")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a local .ai-ledger working directory.
    Init {
        /// Project root where .ai-ledger should be created.
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },

    /// Append or replay local event-log records.
    Event {
        #[command(subcommand)]
        command: EventCommands,
    },
}

#[derive(Debug, Subcommand)]
enum EventCommands {
    /// Append one event to a JSONL event log.
    Append {
        /// JSONL event log path.
        #[arg(long, default_value = ".ai-ledger/events/events.jsonl")]
        log: PathBuf,

        /// Run identifier to store in the event envelope.
        #[arg(long)]
        run_id: String,

        /// Event type for the payload JSON.
        #[arg(long, value_enum, default_value_t = CliEventType::LlmRequest)]
        event_type: CliEventType,

        /// Privacy mode recorded on the event envelope.
        #[arg(long, value_enum, default_value_t = CliPrivacyMode::HashOnly)]
        privacy_mode: CliPrivacyMode,

        /// Payload JSON object. Defaults to {"status":"success"} for llm-request.
        #[arg(long)]
        payload_json: Option<String>,

        /// Envelope tag in key=value form. Can be repeated.
        #[arg(long = "tag", value_parser = parse_tag)]
        tags: Vec<(String, String)>,
    },

    /// Replay a JSONL event log.
    Replay {
        /// JSONL event log path.
        #[arg(long, default_value = ".ai-ledger/events/events.jsonl")]
        log: PathBuf,

        /// Pretty-print events as JSON arrays instead of JSONL.
        #[arg(long)]
        pretty: bool,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliEventType {
    LlmRequest,
    DatasetManifestCreated,
    EvalCaseFailed,
    ReleaseGateFailed,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliPrivacyMode {
    HashOnly,
    MetadataOnly,
    LocalRaw,
    Off,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => init_project(&path),
        Commands::Event { command } => match command {
            EventCommands::Append {
                log,
                run_id,
                event_type,
                privacy_mode,
                payload_json,
                tags,
            } => append_event(
                log,
                run_id,
                event_type,
                privacy_mode.into(),
                payload_json,
                tags,
            ),
            EventCommands::Replay { log, pretty } => replay_events(log, pretty),
        },
    }
}

fn init_project(root: &Path) -> Result<()> {
    let ledger_dir = root.join(".ai-ledger");
    fs::create_dir_all(ledger_dir.join("events")).context("create events directory")?;
    fs::create_dir_all(ledger_dir.join("manifests")).context("create manifests directory")?;
    fs::create_dir_all(ledger_dir.join("reports")).context("create reports directory")?;
    fs::create_dir_all(ledger_dir.join("baselines")).context("create baselines directory")?;

    let config_path = ledger_dir.join("config.toml");
    if !config_path.exists() {
        fs::write(config_path, DEFAULT_CONFIG).context("write default config")?;
    }

    println!("initialized {}", ledger_dir.display());
    Ok(())
}

fn append_event(
    log: PathBuf,
    run_id: String,
    event_type: CliEventType,
    privacy_mode: PrivacyMode,
    payload_json: Option<String>,
    tags: Vec<(String, String)>,
) -> Result<()> {
    if privacy_mode == PrivacyMode::Off {
        println!("event capture disabled by privacy_mode=off");
        return Ok(());
    }

    let payload = payload_from_json(event_type, payload_json)?;
    let mut envelope = EventEnvelope::new(RunId::new(run_id), privacy_mode, payload)?;
    envelope.tags = tags.into_iter().collect::<BTreeMap<_, _>>();

    JsonlEventLog::new(&log)
        .append(&envelope)
        .with_context(|| format!("append event to {}", log.display()))?;

    println!("{}", envelope.event_id);
    Ok(())
}

fn replay_events(log: PathBuf, pretty: bool) -> Result<()> {
    let events = JsonlEventLog::new(&log)
        .replay()
        .with_context(|| format!("replay events from {}", log.display()))?;

    if pretty {
        println!("{}", serde_json::to_string_pretty(&events)?);
    } else {
        for event in events {
            println!("{}", serde_json::to_string(&event)?);
        }
    }

    Ok(())
}

fn payload_from_json(
    event_type: CliEventType,
    payload_json: Option<String>,
) -> Result<EventPayload> {
    let raw = payload_json.unwrap_or_else(|| match event_type {
        CliEventType::LlmRequest => r#"{"status":"success"}"#.to_owned(),
        CliEventType::DatasetManifestCreated => "{}".to_owned(),
        CliEventType::EvalCaseFailed => "{}".to_owned(),
        CliEventType::ReleaseGateFailed => "{}".to_owned(),
    });

    match event_type {
        CliEventType::LlmRequest => Ok(EventPayload::LlmRequest(
            serde_json::from_str::<LlmRequestPayload>(&raw).context("parse llm-request payload")?,
        )),
        CliEventType::DatasetManifestCreated => Ok(EventPayload::DatasetManifestCreated(
            serde_json::from_str::<DatasetManifestCreatedPayload>(&raw)
                .context("parse dataset-manifest-created payload")?,
        )),
        CliEventType::EvalCaseFailed => Ok(EventPayload::EvalCaseFailed(
            serde_json::from_str::<EvalCaseFailedPayload>(&raw)
                .context("parse eval-case-failed payload")?,
        )),
        CliEventType::ReleaseGateFailed => Ok(EventPayload::ReleaseGateFailed(
            serde_json::from_str::<ReleaseGateFailedPayload>(&raw)
                .or_else(|_| parse_release_gate_without_failures(&raw))
                .context("parse release-gate-failed payload")?,
        )),
    }
}

fn parse_release_gate_without_failures(raw: &str) -> serde_json::Result<ReleaseGateFailedPayload> {
    #[derive(serde::Deserialize)]
    struct Minimal {
        policy_id: String,
        candidate_report: String,
        baseline_report: String,
    }

    let minimal = serde_json::from_str::<Minimal>(raw)?;
    Ok(ReleaseGateFailedPayload {
        policy_id: minimal.policy_id,
        candidate_report: minimal.candidate_report,
        baseline_report: minimal.baseline_report,
        failures: Vec::<GateFailure>::new(),
    })
}

fn parse_tag(value: &str) -> Result<(String, String), String> {
    let (key, val) = value
        .split_once('=')
        .ok_or_else(|| "tags must use key=value form".to_owned())?;

    if key.trim().is_empty() {
        return Err("tag key must not be empty".to_owned());
    }

    Ok((key.to_owned(), val.to_owned()))
}

impl From<CliPrivacyMode> for PrivacyMode {
    fn from(value: CliPrivacyMode) -> Self {
        match value {
            CliPrivacyMode::HashOnly => Self::HashOnly,
            CliPrivacyMode::MetadataOnly => Self::MetadataOnly,
            CliPrivacyMode::LocalRaw => Self::LocalRaw,
            CliPrivacyMode::Off => Self::Off,
        }
    }
}

const DEFAULT_CONFIG: &str = r#"[project]
name = "example-rag-app"
environment = "dev"

[privacy]
mode = "hash_only"
hash_algorithm = "blake3"
allow_raw_prompts = false
allow_raw_outputs = false
allow_raw_rag_context = false
redact_file_paths = true
redact_hostnames = true

[event_log]
format = "jsonl"
path = ".ai-ledger/events/events.jsonl"
flush_policy = "line"
rotate_after_mb = 256

[export.splunk]
enabled = false

[export.otel]
enabled = false
"#;
