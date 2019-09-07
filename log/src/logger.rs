//! # Logger
//!
//! `logger` is the module containing the logger type and functions.

use crate::file::LogFile;
use crate::format::LogFormat;
use crate::level::LogLevel;
use crate::result::Result;
use config::log_config::LogConfig;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{stderr, stdout, Write};

/// `write_to_stdout` writes a binary message to stdout.
fn write_to_stdout(msg: &[u8]) -> Result<()> {
    stdout().write_all(msg).map_err(|e| e.into())
}

/// `write_to_stderr` writes a binary message to stderr.
fn write_to_stderr(msg: &[u8]) -> Result<()> {
    stderr().write_all(msg).map_err(|e| e.into())
}

/// `write_to_file` writes a binary message to a regular file.
/// The file is created if missing.
fn write_to_file(path: &str, msg: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    file.write_all(msg)?;
    file.write_all(b"\n").map_err(|e| e.into())
}

/// `write_with_log_file` writes a binary message using a given `LogFile`.
fn write_with_log_file(file: &LogFile, msg: &[u8]) -> Result<()> {
    match file {
        LogFile::StdOut => write_to_stdout(msg),
        LogFile::StdErr => write_to_stderr(msg),
        LogFile::Path(path) => write_to_file(&path, msg),
    }
}

/// `Logger` is the logger type used in Alsacoin.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash, Serialize, Deserialize)]
pub struct Logger {
    level: LogLevel,
    format: LogFormat,
    file: LogFile,
}

impl Logger {
    /// `new` creates a new `Logger`.
    pub fn new(level: LogLevel, format: LogFormat, file: &LogFile) -> Logger {
        Logger {
            level,
            format,
            file: file.to_owned(),
        }
    }

    /// `from_config` creates a new `Logger` from a `LogConfig`.
    pub fn from_config(config: &LogConfig) -> Result<Logger> {
        config.validate()?;

        let mut config = config.clone();
        config.populate();

        let level = LogLevel::parse(&config.level.unwrap())?;
        let format = LogFormat::parse(&config.format.unwrap())?;
        let file = LogFile::parse(&config.file.unwrap());

        let logger = Logger {
            level,
            format,
            file,
        };

        Ok(logger)
    }
}

#[test]
fn test_logger_from_config() {
    let valid_config = LogConfig::default();
    let mut invalid_config = valid_config.clone();
    invalid_config.level = Some("level".into());

    let res = Logger::from_config(&invalid_config);
    assert!(res.is_err());

    let res = Logger::from_config(&valid_config);
    assert!(res.is_ok());
}
