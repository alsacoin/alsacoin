//! # Log Level
//!
//! `level` is the module containing the log level type and functions.

use crate::error::Error;
use crate::result::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

/// `LogLevel` represents the level of the output of a log operation.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    None,
    Critical,
    Info,
    Debug,
}

impl LogLevel {
    /// Parses a `LogLevel` from a `&str`.
    pub fn parse(s: &str) -> Result<LogLevel> {
        match s {
            "none" => Ok(LogLevel::None),
            "critical" => Ok(LogLevel::Critical),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            _ => {
                let err = Error::Parse {
                    msg: "invalid level".into(),
                };
                Err(err)
            }
        }
    }

    /// `is_none` returns if the record is a `None` `LogLevel`.
    pub fn is_none(self) -> bool {
        match self {
            LogLevel::None => true,
            _ => false,
        }
    }

    /// `is_critical` returns if the record is a `Critical` `LogLevel`.
    pub fn is_critical(self) -> bool {
        match self {
            LogLevel::Critical => true,
            _ => false,
        }
    }

    /// `is_info` returns if the record is a `Info` `LogLevel`.
    pub fn is_info(self) -> bool {
        match self {
            LogLevel::Info => true,
            _ => false,
        }
    }

    /// `is_debug` returns if the record is a `Debug` `LogLevel`.
    pub fn is_debug(self) -> bool {
        match self {
            LogLevel::Debug => true,
            _ => false,
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LogLevel::None => write!(f, "none"),
            LogLevel::Critical => write!(f, "critical"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Debug => write!(f, "debug"),
        }
    }
}

impl Default for LogLevel {
    fn default() -> LogLevel {
        LogLevel::None
    }
}

#[test]
fn test_level_parse() {
    let valid_level_a = "debug";

    let res = LogLevel::parse(valid_level_a);
    assert!(res.is_ok());

    let valid_level_b = res.unwrap();
    assert_eq!(valid_level_a, format!("{}", valid_level_b));

    let invalid_level = "level";

    let res = LogLevel::parse(invalid_level);
    assert!(res.is_err());
}
