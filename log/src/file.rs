//! # Log File
//!
//! `file` is the module containing the log file type and functions.

use serde::{Deserialize, Serialize};
use std::fmt;

/// `LogFile` represents the output file of a log operation.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub enum LogFile {
    StdOut,
    StdErr,
    Path(String),
}

impl LogFile {
    /// Parses a `LogFile` from a `&str`.
    pub fn parse(s: &str) -> LogFile {
        match s {
            "stdout" => LogFile::StdOut,
            "stderr" => LogFile::StdErr,
            path => LogFile::Path(path.into()),
        }
    }

    /// `is_stdout` returns if it is a `StdOut` `LogFile`.
    pub fn is_stdout(&self) -> bool {
        match self {
            LogFile::StdOut => true,
            _ => false,
        }
    }

    /// `is_stderr` returns if it is a `StdErr` `LogFile`.
    pub fn is_stderr(&self) -> bool {
        match self {
            LogFile::StdErr => true,
            _ => false,
        }
    }

    /// `is_path` returns if it is a `Path` `LogFile`.
    pub fn is_path(&self) -> bool {
        match self {
            LogFile::Path(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for LogFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LogFile::StdOut => write!(f, "stdout"),
            LogFile::StdErr => write!(f, "stderr"),
            LogFile::Path(path) => write!(f, "{}", path),
        }
    }
}

impl Default for LogFile {
    fn default() -> LogFile {
        LogFile::StdErr
    }
}
