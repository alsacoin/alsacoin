//! # LogLevel
//!
//! `file` is the module containing the log level type and functions.

use crate::error::Error;
use crate::result::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

/// `LogLevel` represents the level of the output of a log operation.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Critical,
}

impl LogLevel {
    /// Parses a `LogLevel` from a `&str`.
    pub fn parse(s: &str) -> Result<LogLevel> {
        match s {
            "debug" => Ok(LogLevel::Debug),
            "critical" => Ok(LogLevel::Critical),
            _ => {
                let err = Error::Parse {
                    msg: "invalid level".into(),
                };
                Err(err)
            }
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Critical => write!(f, "critical"),
        }
    }
}

impl Default for LogLevel {
    fn default() -> LogLevel {
        LogLevel::Critical
    }
}
