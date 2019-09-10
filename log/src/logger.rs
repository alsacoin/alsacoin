//! # Logger
//!
//! `logger` is the module containing the logger type and functions.

use crate::color::LogColor;
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
use term;

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

/// `Logger` is the logger type used in Alsacoin.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash, Serialize, Deserialize)]
pub struct Logger {
    level: LogLevel,
    format: LogFormat,
    file: LogFile,
    color: bool,
}

impl Logger {
    /// `new` creates a new `Logger`. The logger logs
    /// in json or binary or string on stderr and stdout,
    /// but only in json and binary on file.
    pub fn new(level: LogLevel, format: LogFormat, file: &LogFile, color: bool) -> Result<Logger> {
        if file.is_path() && format.is_raw() {
            let err = Error::InvalidFormat;
            return Err(err);
        }

        let logger = Logger {
            level,
            format,
            file: file.to_owned(),
            color,
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
        let color = config.color.unwrap();

        let logger = Logger {
            level,
            format,
            file,
            color,
        };

        Ok(logger)
    }

    /// `log_record` returns a `LogRecord` from a log message.
    pub fn log_record(level: LogLevel, msg: &str) -> Result<LogRecord> {
        if level.is_none() {
            let err = Error::InvalidLevel;
            return Err(err);
        }

        LogRecord::new(level, msg)
    }

    /// `log_message` returns the binary log message from a string message.
    pub fn log_message(level: LogLevel, format: LogFormat, msg: &str) -> Result<Vec<u8>> {
        if level.is_none() {
            let err = Error::InvalidLevel;
            return Err(err);
        }

        let record = Logger::log_record(level, msg)?;

        let msg = match format {
            LogFormat::Raw => record.to_string().into_bytes(),
            LogFormat::JSON => record.to_json()?.into_bytes(),
        };

        Ok(msg)
    }

    /// `log_to_file` logs a message on a file.
    pub fn log_to_file(path: &str, level: LogLevel, format: LogFormat, msg: &str) -> Result<()> {
        if level.is_none() {
            let err = Error::InvalidLevel;
            return Err(err);
        }

        let msg = Logger::log_message(level, format, msg)?;

        write_to_file(path, &msg)
    }

    /// `log_to_stdout` logs a message on stdout. It does nothing if it should not.
    pub fn log_to_stdout(level: LogLevel, format: LogFormat, color: bool, msg: &str) -> Result<()> {
        if level.is_none() {
            let err = Error::InvalidLevel;
            return Err(err);
        }

        let msg = Logger::log_message(level, format, msg)?;

        if color {
            let mut t = match term::stdout() {
                Some(t) => t,
                None => {
                    let err = Error::NotFound;
                    return Err(err);
                }
            };

            match LogColor::level_color(level) {
                LogColor::None => {}
                LogColor::Red => {
                    t.fg(term::color::RED)?;
                }
                LogColor::Blue => {
                    t.fg(term::color::BLUE)?;
                }
                LogColor::Green => {
                    t.fg(term::color::GREEN)?;
                }
            }

            write_to_stdout(&msg)?;

            t.reset()?;
        } else {
            write_to_stdout(&msg)?;
        }

        Ok(())
    }

    /// `log_to_stderr` logs a message on stderr
    pub fn log_to_stderr(level: LogLevel, format: LogFormat, color: bool, msg: &str) -> Result<()> {
        if level.is_none() {
            let err = Error::InvalidLevel;
            return Err(err);
        }

        let msg = Logger::log_message(level, format, msg)?;

        if color {
            let mut t = match term::stdout() {
                Some(t) => t,
                None => {
                    let err = Error::NotFound;
                    return Err(err);
                }
            };

            match LogColor::level_color(level) {
                LogColor::None => {}
                LogColor::Red => {
                    t.fg(term::color::RED)?;
                }
                LogColor::Blue => {
                    t.fg(term::color::BLUE)?;
                }
                LogColor::Green => {
                    t.fg(term::color::GREEN)?;
                }
            }

            write_to_stderr(&msg)?;

            t.reset()?;
        } else {
            write_to_stderr(&msg)?;
        }

        Ok(())
    }

    /// `log` logs a message at a specific level. If the given
    /// level is greater than the logger level, the logger does
    /// nothing.
    pub fn log(&self, level: LogLevel, msg: &str) -> Result<()> {
        if self.level.is_none() || self.level < level {
            return Ok(());
        }

        match self.file {
            LogFile::StdOut => Logger::log_to_stdout(level, self.format, self.color, msg),
            LogFile::StdErr => Logger::log_to_stderr(level, self.format, self.color, msg),
            LogFile::Path(ref path) => Logger::log_to_file(path, level, self.format, msg),
        }
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
    let color = true;

    let res = Logger::new(level, format, &file, color);
    assert!(res.is_ok());

    let format = LogFormat::Raw;
    let file = LogFile::Path("path".into());

    let res = Logger::new(level, format, &file, color);
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
    assert!(res.is_ok());
}

#[test]
fn test_logger_log_record() {
    let valid_msg = "abcd";
    let invalid_msg = "\n";

    let level = LogLevel::Info;

    let res = Logger::log_record(level, invalid_msg);
    assert!(res.is_err());

    let res = Logger::log_record(level, valid_msg);
    assert!(res.is_ok());
}

#[test]
fn test_logger_log_message() {
    // let valid_msg = "abcd";
    let invalid_msg = "\n";

    let level = LogLevel::Info;
    let format = LogFormat::default();

    let res = Logger::log_message(level, format, invalid_msg);
    assert!(res.is_err());

    /*
    // TODO: fatal runtime error: stack overflow
    let res = Logger::log_message(level, format, valid_msg);
    assert!(res.is_ok());
    */
}
