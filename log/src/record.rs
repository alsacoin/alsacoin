//! # Log Record
//!
//! `record` is the module containing the log record type and functions.

use crate::error::Error;
use crate::level::LogLevel;
use crate::result::Result;
use models::timestamp::Timestamp;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use std::fmt;

/// `LogRecord` is the log record type.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug, Hash, Serialize, Deserialize)]
pub struct LogRecord {
    pub timestamp: Timestamp,
    pub level: LogLevel,
    pub content: String,
}

impl LogRecord {
    /// `new` creates a new `LogRecord`.
    pub fn new(level: LogLevel, content: &str) -> Result<LogRecord> {
        if !content.is_ascii() || content.contains("\n") {
            let err = Error::InvalidFormat;
            return Err(err);
        }

        let record = LogRecord {
            timestamp: Timestamp::now(),
            level,
            content: content.into(),
        };

        Ok(record)
    }

    /// `validate` validates the `LogRecord`.
    pub fn validate(&self) -> Result<()> {
        self.timestamp.validate()?;

        if !self.content.is_ascii() || self.content.contains("\n") {
            let err = Error::InvalidFormat;
            return Err(err);
        }

        Ok(())
    }

    /// `to_bytes` converts the `LogRecord` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `LogRecord`.
    pub fn from_bytes(b: &[u8]) -> Result<LogRecord> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `LogRecord` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `LogRecord`.
    pub fn from_json(s: &str) -> Result<LogRecord> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

impl fmt::Display for LogRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Time: {}, level: {}, {}",
            self.timestamp, self.level, self.content
        )
    }
}

#[test]
fn test_log_new() {
    let level = LogLevel::default();
    let valid_content = "";
    let invalid_content = "❤";

    let res = LogRecord::new(level, invalid_content);
    assert!(res.is_err());

    let res = LogRecord::new(level, valid_content);
    assert!(res.is_ok());
}

#[test]
fn test_log_validate() {
    let date = "2012-12-12T00:00:00Z";
    let invalid_timestamp = Timestamp::parse(date).unwrap();
    let invalid_content = "\n";

    let mut record = LogRecord::default();
    let res = record.validate();
    assert!(res.is_ok());

    record.timestamp = invalid_timestamp;
    let res = record.validate();
    assert!(res.is_err());

    record.timestamp = Timestamp::now();

    record.content = invalid_content.into();
    let res = record.validate();
    assert!(res.is_err());
}

#[test]
fn test_record_serialize_bytes() {
    let record_a = LogRecord::default();

    let res = record_a.to_bytes();
    assert!(res.is_ok());
    let cbor = res.unwrap();

    let res = LogRecord::from_bytes(&cbor);
    assert!(res.is_ok());
    let record_b = res.unwrap();

    assert_eq!(record_a, record_b)
}

#[test]
fn test_record_serialize_json() {
    let record_a = LogRecord::default();

    let res = record_a.to_json();
    assert!(res.is_ok());
    let json = res.unwrap();

    let res = LogRecord::from_json(&json);
    assert!(res.is_ok());
    let record_b = res.unwrap();

    assert_eq!(record_a, record_b)
}
