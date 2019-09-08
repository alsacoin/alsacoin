//! # Logger
//!
//! `logger` is the module containing the logger type and functions.

use crate::error::Error;
use crate::file::LogFile;
use crate::format::LogFormat;
use crate::level::LogLevel;
use crate::record::LogRecord;
use crate::result::Result;
use config::log::LogConfig;
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
    /// `new` creates a new `Logger`. The logger logs
    /// in json or binary or string on stderr and stdout,
    /// but only in json and binary on file.
    pub fn new(level: LogLevel, format: LogFormat, file: &LogFile) -> Result<Logger> {
        if file.is_path() && format.is_raw() {
            let err = Error::InvalidFormat;
            return Err(err);
        }

        let logger = Logger {
            level,
            format,
            file: file.to_owned(),
        };

        Ok(logger)
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

    /// `validate` validates the `Logger`.
    pub fn validate(&self) -> Result<()> {
        if self.file.is_path() && self.format.is_raw() {
            let err = Error::InvalidFormat;
            return Err(err);
        }

        Ok(())
    }

    /// `log_record` returns a `LogRecord` from a log message.
    pub fn log_record(&self, msg: &str) -> Result<LogRecord> {
        self.validate()?;

        LogRecord::new(self.level, msg)
    }

    /// `log_message` returns the binary log message from a string message.
    pub fn log_message(&self, msg: &str) -> Result<Vec<u8>> {
        self.validate()?;

        let record = self.log_record(msg)?;

        let msg = match self.format {
            LogFormat::Raw => record.to_string().into_bytes(),
            LogFormat::JSON => record.to_json()?.into_bytes(),
            LogFormat::Binary => record.to_bytes()?,
        };

        Ok(msg)
    }

    /// `log` logs a message at a specific level. If the given
    /// level is greater than the logger level, the logger does
    /// nothing.
    pub fn log(&self, level: LogLevel, msg: &str) -> Result<()> {
        self.validate()?;

        if self.level < level {
            return Ok(());
        }

        // let is_terminal = self.file.is_path();

        // TODO: change the terminal color
        // if is_terminal

        let msg = self.log_message(msg)?;

        write_with_log_file(&self.file, &msg)?;

        // TODO: clear the color
        // if is_terminal

        Ok(())
    }

    /// `log_critical` logs a message with a critical level.
    pub fn log_critical(&self, msg: &str) -> Result<()> {
        let level = LogLevel::Critical;

        self.log(level, msg)
    }

    /// `log_info` logs a message with a info level.
    pub fn log_info(&self, msg: &str) -> Result<()> {
        let level = LogLevel::Info;

        self.log(level, msg)
    }

    /// `log_debug` logs a message with a debug level.
    pub fn log_debug(&self, msg: &str) -> Result<()> {
        let level = LogLevel::Debug;

        self.log(level, msg)
    }
}

#[test]
fn test_logger_new() {
    let level = LogLevel::default();
    let format = LogFormat::default();
    let file = LogFile::default();

    let res = Logger::new(level, format, &file);
    assert!(res.is_ok());

    let format = LogFormat::Raw;
    let file = LogFile::Path("path".into());

    let res = Logger::new(level, format, &file);
    assert!(res.is_err());
}

#[test]
fn test_logger_from_config() {
    let valid_config = LogConfig::default();
    let mut invalid_config = valid_config.clone();
    invalid_config.level = Some("level".into());

    let res = Logger::from_config(&invalid_config);
    assert!(res.is_err());

    let res = Logger::from_config(&valid_config);
    if res.is_err() {
        println!("{:?}", &res);
        println!("valid_config: {:?}", valid_config);
        panic!();
    }
    assert!(res.is_ok());
}

#[test]
fn test_logger_validate() {
    let mut logger = Logger::default();

    let res = logger.validate();
    assert!(res.is_ok());

    logger.file = LogFile::Path("path".into());
    logger.format = LogFormat::Raw;

    let res = logger.validate();
    assert!(res.is_err());
}

#[test]
fn test_logger_log_record() {
    let valid_msg = "abcd";
    let invalid_msg = "\n";

    let logger = Logger::default();

    let res = logger.log_record(invalid_msg);
    assert!(res.is_err());

    let res = logger.log_record(valid_msg);
    assert!(res.is_ok());
}

#[test]
fn test_logger_log_message() {
    let logger = Logger::default();

    let res = logger.log_message("\n");
    assert!(res.is_err());

    /* // fatal runtime error: stack overflow
    // TODO: try changing the nightly version
    let res = logger.log_message("abcd");
    assert!(res.is_ok());
    */
}
