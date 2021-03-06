//! # Log Format
//!
//! `format` is the module containing the log format type and functions.

use crate::error::Error;
use crate::result::Result;
use std::fmt;

/// `LogFormat` represents the format of the output of a log operation.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum LogFormat {
    Raw,
    JSON,
}

impl LogFormat {
    /// Parses a `LogFormat` from a `&str`.
    pub fn parse(s: &str) -> Result<LogFormat> {
        match s {
            "raw" => Ok(LogFormat::Raw),
            "json" => Ok(LogFormat::JSON),
            _ => {
                let err = Error::Parse {
                    msg: "invalid format".into(),
                };
                Err(err)
            }
        }
    }

    /// `is_raw` returns if it is a `Raw` `LogFormat`.
    pub fn is_raw(self) -> bool {
        match self {
            LogFormat::Raw => true,
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
}

impl fmt::Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LogFormat::Raw => write!(f, "raw"),
            LogFormat::JSON => write!(f, "json"),
        }
    }
}

impl Default for LogFormat {
    fn default() -> LogFormat {
        LogFormat::Raw
    }
}

#[test]
fn test_format_parse() {
    let valid_format_a = "json";

    let res = LogFormat::parse(valid_format_a);
    assert!(res.is_ok());

    let valid_format_b = res.unwrap();
    assert_eq!(valid_format_a, format!("{}", valid_format_b));

    let invalid_format = "format";

    let res = LogFormat::parse(invalid_format);
    assert!(res.is_err());
}
