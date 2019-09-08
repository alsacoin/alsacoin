//! # Log Color
//!
//! `color` is the module containing the log color type and functions.

use crate::error::Error;
use crate::level::LogLevel;
use crate::result::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

/// `LogColor` represents the color of the output of a log operation.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum LogColor {
    Red,
    Blue,
    Green,
}

impl LogColor {
    /// Parses a `LogColor` from a `&str`.
    pub fn parse(s: &str) -> Result<LogColor> {
        match s {
            "red" => Ok(LogColor::Red),
            "blue" => Ok(LogColor::Blue),
            "green" => Ok(LogColor::Green),
            _ => {
                let err = Error::Parse {
                    msg: "invalid color".into(),
                };
                Err(err)
            }
        }
    }

    /// `level_color` returns a `LogLevel` `LogColor`.
    pub fn level_color(level: LogLevel) -> LogColor {
        match level {
            LogLevel::Critical => LogColor::Red,
            LogLevel::Info => LogColor::Blue,
            LogLevel::Debug => LogColor::Green,
        }
    }

    /// `is_red` returns if the record is a `Red` `LogColor`.
    pub fn is_red(self) -> bool {
        match self {
            LogColor::Red => true,
            _ => false,
        }
    }

    /// `is_blue` returns if the record is a `Blue` `LogColor`.
    pub fn is_blue(self) -> bool {
        match self {
            LogColor::Blue => true,
            _ => false,
        }
    }

    /// `is_green` returns if the record is a `Green` `LogColor`.
    pub fn is_green(self) -> bool {
        match self {
            LogColor::Green => true,
            _ => false,
        }
    }
}

impl fmt::Display for LogColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LogColor::Red => write!(f, "red"),
            LogColor::Blue => write!(f, "blue"),
            LogColor::Green => write!(f, "green"),
        }
    }
}

impl Default for LogColor {
    fn default() -> LogColor {
        LogColor::Red
    }
}

#[test]
fn test_color_parse() {
    let valid_color_a = "green";

    let res = LogColor::parse(valid_color_a);
    assert!(res.is_ok());

    let valid_color_b = res.unwrap();
    assert_eq!(valid_color_a, format!("{}", valid_color_b));

    let invalid_color = "color";

    let res = LogColor::parse(invalid_color);
    assert!(res.is_err());
}
