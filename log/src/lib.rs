//! # Log
//!
//! `log` contains Alsacoin`s logging types and functions.

#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `file` contains the log file type and functions.
pub mod file;
