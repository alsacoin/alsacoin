//! # Log
//!
//! `log` contains Alsacoin`s logging types and functions.

#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `level` contains the log level type and functions.
pub mod level;

/// `format` contains the log format type and functions.
pub mod format;

/// `file` contains the log file type and functions.
pub mod file;

/// `record` contains the log record type and functions.
pub mod record;

/// `logger` contains the logger type and functions.
pub mod logger;
