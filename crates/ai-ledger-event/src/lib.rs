use ai_ledger_core::{AiLedgerError, EventId, PrivacyMode, Result, RunId, timestamp_ns};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SCHEMA_VERSION: &str = "0.1.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    LlmRequest,
    DatasetManifestCreated,
    EvalCaseFailed,
    ReleaseGateFailed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: EventId,
    pub event_type: EventType,
    pub schema_version: String,
    pub timestamp_ns: u64,
    pub run_id: RunId,
    pub privacy_mode: PrivacyMode,
    pub payload: EventPayload,
    #[serde(default)]
    pub tags: BTreeMap<String, String>,
}

impl EventEnvelope {
    pub fn new(run_id: RunId, privacy_mode: PrivacyMode, payload: EventPayload) -> Result<Self> {
        let event_type = payload.event_type();
        let envelope = Self {
            event_id: EventId::new(),
            event_type,
            schema_version: SCHEMA_VERSION.to_owned(),
            timestamp_ns: timestamp_ns(),
            run_id,
            privacy_mode,
            payload,
            tags: BTreeMap::new(),
        };
        envelope.validate()?;
        Ok(envelope)
    }

    pub fn validate(&self) -> Result<()> {
        self.run_id.validate()?;

        if self.schema_version != SCHEMA_VERSION {
            return Err(AiLedgerError::InvalidEvent {
                reason: format!(
                    "schema_version must be {SCHEMA_VERSION}, got {}",
                    self.schema_version
                ),
            });
        }

        if self.timestamp_ns == 0 {
            return Err(AiLedgerError::InvalidEvent {
                reason: "timestamp_ns must be non-zero".to_owned(),
            });
        }

        if self.event_type != self.payload.event_type() {
            return Err(AiLedgerError::InvalidEvent {
                reason: "event_type does not match payload".to_owned(),
            });
        }

        self.payload.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventPayload {
    LlmRequest(LlmRequestPayload),
    DatasetManifestCreated(DatasetManifestCreatedPayload),
    EvalCaseFailed(EvalCaseFailedPayload),
    ReleaseGateFailed(ReleaseGateFailedPayload),
}

impl EventPayload {
    pub fn event_type(&self) -> EventType {
        match self {
            Self::LlmRequest(_) => EventType::LlmRequest,
            Self::DatasetManifestCreated(_) => EventType::DatasetManifestCreated,
            Self::EvalCaseFailed(_) => EventType::EvalCaseFailed,
            Self::ReleaseGateFailed(_) => EventType::ReleaseGateFailed,
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::LlmRequest(payload) => require_non_empty("payload.status", &payload.status),
            Self::DatasetManifestCreated(payload) => {
                require_non_empty("payload.dataset_name", &payload.dataset_name)?;
                require_non_empty("payload.dataset_hash", &payload.dataset_hash)
            }
            Self::EvalCaseFailed(payload) => {
                require_non_empty("payload.suite_id", &payload.suite_id)?;
                require_non_empty("payload.case_id", &payload.case_id)?;
                require_non_empty("payload.severity", &payload.severity)?;
                require_non_empty("payload.reason", &payload.reason)
            }
            Self::ReleaseGateFailed(payload) => {
                require_non_empty("payload.policy_id", &payload.policy_id)?;
                require_non_empty("payload.candidate_report", &payload.candidate_report)?;
                require_non_empty("payload.baseline_report", &payload.baseline_report)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LlmRequestPayload {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rag_context_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_input: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_output: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DatasetManifestCreatedPayload {
    pub dataset_name: String,
    pub dataset_hash: String,
    pub file_count: u64,
    pub total_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvalCaseFailedPayload {
    pub suite_id: String,
    pub case_id: String,
    pub severity: String,
    pub reason: String,
    pub input_hash: String,
    pub output_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReleaseGateFailedPayload {
    pub policy_id: String,
    pub candidate_report: String,
    pub baseline_report: String,
    #[serde(default)]
    pub failures: Vec<GateFailure>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GateFailure {
    pub metric: String,
    pub operator: String,
    pub threshold: String,
    pub actual: String,
}

fn require_non_empty(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(AiLedgerError::InvalidEvent {
            reason: format!("{field} must not be empty"),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llm_request_serializes_stable_event_fields() {
        let event = EventEnvelope::new(
            RunId::new("run_abc"),
            PrivacyMode::HashOnly,
            EventPayload::LlmRequest(LlmRequestPayload {
                status: "success".to_owned(),
                model_hash: Some("sha256:model".to_owned()),
                prompt_hash: Some("sha256:prompt".to_owned()),
                rag_context_hash: None,
                latency_ms: Some(823),
                token_input: Some(1840),
                token_output: Some(312),
            }),
        )
        .expect("valid event");

        let json = serde_json::to_value(event).expect("serialize event");
        assert_eq!(json["event_type"], "llm_request");
        assert_eq!(json["schema_version"], SCHEMA_VERSION);
        assert_eq!(json["privacy_mode"], "hash_only");
        assert_eq!(json["payload"]["kind"], "llm_request");
        assert_eq!(json["payload"]["prompt_hash"], "sha256:prompt");
    }

    #[test]
    fn validation_rejects_missing_required_payload_field() {
        let err = EventEnvelope::new(
            RunId::new("run_abc"),
            PrivacyMode::HashOnly,
            EventPayload::LlmRequest(LlmRequestPayload {
                status: " ".to_owned(),
                model_hash: None,
                prompt_hash: None,
                rag_context_hash: None,
                latency_ms: None,
                token_input: None,
                token_output: None,
            }),
        )
        .expect_err("invalid event");

        assert!(err.to_string().contains("payload.status"));
    }
}
