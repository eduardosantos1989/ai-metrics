use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, AiLedgerError>;

#[derive(Debug, Error)]
pub enum AiLedgerError {
    #[error("invalid configuration for {field}: {reason}")]
    InvalidConfig { field: String, reason: String },

    #[error("invalid event: {reason}")]
    InvalidEvent { reason: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventId(String);

impl EventId {
    pub fn new() -> Self {
        Self(Uuid::now_v7().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RunId(String);

impl RunId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn validate(&self) -> Result<()> {
        if self.0.trim().is_empty() {
            return Err(AiLedgerError::InvalidEvent {
                reason: "run_id must not be empty".to_owned(),
            });
        }

        Ok(())
    }
}

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

id_type!(ModelId);
id_type!(DatasetId);
id_type!(ConfigHash);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyMode {
    HashOnly,
    MetadataOnly,
    LocalRaw,
    Off,
}

impl Default for PrivacyMode {
    fn default() -> Self {
        Self::HashOnly
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HashAlgorithm {
    Blake3,
    Sha256,
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        Self::Blake3
    }
}

pub fn timestamp_ns() -> u64 {
    time::OffsetDateTime::now_utc()
        .unix_timestamp_nanos()
        .try_into()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privacy_mode_uses_snake_case_serialization() {
        let encoded = serde_json::to_string(&PrivacyMode::HashOnly).expect("serialize privacy");
        assert_eq!(encoded, "\"hash_only\"");
    }

    #[test]
    fn invalid_privacy_mode_fails_deserialization() {
        let err = serde_json::from_str::<PrivacyMode>("\"raw\"").expect_err("invalid mode");
        assert!(err.to_string().contains("unknown variant"));
    }

    #[test]
    fn run_id_rejects_empty_values() {
        let err = RunId::new(" ").validate().expect_err("empty run id");
        assert!(err.to_string().contains("run_id"));
    }
}
