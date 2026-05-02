use ai_ledger_core::{AiLedgerError, Result};
use ai_ledger_event::EventEnvelope;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::{BufRead, BufReader, Lines, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct JsonlEventLog {
    path: PathBuf,
}

impl JsonlEventLog {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(&self, event: &EventEnvelope) -> Result<()> {
        event.validate()?;

        if let Some(parent) = self.path.parent() {
            create_dir_all(parent)?;
        }

        let mut line = serde_json::to_vec(event)?;
        line.push(b'\n');

        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?
            .write_all(&line)?;
        Ok(())
    }

    pub fn replay(&self) -> Result<JsonlEventIter> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        Ok(JsonlEventIter {
            lines: reader.lines(),
            next_line_number: 0,
        })
    }
}

pub struct JsonlEventIter {
    lines: Lines<BufReader<File>>,
    next_line_number: usize,
}

impl Iterator for JsonlEventIter {
    type Item = Result<EventEnvelope>;

    fn next(&mut self) -> Option<Self::Item> {
        for line in self.lines.by_ref() {
            self.next_line_number += 1;
            let line = match line {
                Ok(line) => line,
                Err(source) => return Some(Err(AiLedgerError::Io(source))),
            };
            if line.trim().is_empty() {
                continue;
            }

            let event = serde_json::from_str::<EventEnvelope>(&line).map_err(|source| {
                AiLedgerError::InvalidEvent {
                    reason: format!(
                        "malformed JSONL record on line {}: {source}",
                        self.next_line_number
                    ),
                }
            });

            return Some(event.and_then(|event| {
                event.validate()?;
                Ok(event)
            }));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_ledger_core::{PrivacyMode, RunId};
    use ai_ledger_event::{EventEnvelope, EventPayload, LlmRequestPayload};
    use std::fs;

    fn sample_event() -> EventEnvelope {
        EventEnvelope::new(
            RunId::new("run_test"),
            PrivacyMode::HashOnly,
            EventPayload::LlmRequest(LlmRequestPayload {
                status: "success".to_owned(),
                model_hash: Some("sha256:model".to_owned()),
                prompt_hash: Some("sha256:prompt".to_owned()),
                rag_context_hash: None,
                latency_ms: Some(1),
                token_input: None,
                token_output: None,
            }),
        )
        .expect("sample event")
    }

    #[test]
    fn append_and_replay_round_trip_events() {
        let dir = tempfile::tempdir().expect("tempdir");
        let log = JsonlEventLog::new(dir.path().join("events/events.jsonl"));

        log.append(&sample_event()).expect("append first");
        log.append(&sample_event()).expect("append second");

        let events = log
            .replay()
            .expect("open replay iterator")
            .collect::<Result<Vec<_>>>()
            .expect("replay events");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, ai_ledger_event::EventType::LlmRequest);
    }

    #[test]
    fn malformed_records_return_line_number() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("events.jsonl");
        fs::write(&path, "{bad json}\n").expect("write malformed");

        let err = JsonlEventLog::new(path)
            .replay()
            .expect("open replay iterator")
            .collect::<Result<Vec<_>>>()
            .expect_err("malformed");
        assert!(err.to_string().contains("line 1"));
    }
}
