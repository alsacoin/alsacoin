//! # Log Format
//!
//! `format` is the module containing the log format type and functions.

use crate::error::Error;
use crate::result::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

/// `LogFormat` represents the format of the output of a log operation.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum LogFormat {
    String,
    JSON,
    Binary,
}

impl LogFormat {
    /// Parses a `LogFormat` from a `&str`.
    pub fn parse(s: &str) -> Result<LogFormat> {
        match s {
            "raw" => Ok(LogFormat::String),
            "json" => Ok(LogFormat::JSON),
            "binary" => Ok(LogFormat::Binary),
            _ => {
                let err = Error::Parse {
                    msg: "invalid format".into(),
                };
                Err(err)
            }
        }
    }

    /// `is_string` returns if it is a `String` `LogFormat`.
    pub fn is_string(self) -> bool {
        match self {
            LogFormat::String => true,
            _ => false,
        }
    }

    /// `is_json` returns if it is a `JSON` `LogFormat`.
    pub fn is_json(self) -> bool {
        match self {
            LogFormat::JSON => true,
            _ => false,
        }
    }

    /// `is_binary` returns if it is a `Binary` `LogFormat`.
    pub fn is_binary(self) -> bool {
        match self {
            LogFormat::Binary => true,
            _ => false,
        }
    }
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LogFormat::String => write!(f, "raw"),
            LogFormat::JSON => write!(f, "json"),
            LogFormat::Binary => write!(f, "binary"),
        }
    }
}

impl Default for LogFormat {
    fn default() -> LogFormat {
        LogFormat::String
    }
}

#[test]
fn test_format_parse() {
    let valid_format_a = "binary";

    let res = LogFormat::parse(valid_format_a);
    assert!(res.is_ok());

    let valid_format_b = res.unwrap();
    assert_eq!(valid_format_a, format!("{}", valid_format_b));

    let invalid_format = "format";

    let res = LogFormat::parse(invalid_format);
    assert!(res.is_err());
}
