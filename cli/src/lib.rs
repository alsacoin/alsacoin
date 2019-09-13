//! # CLI
//!
//! `cli` contains Alsacoin`s CLI types and functions.

#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `common` contains the crate common functionalities.
pub mod common;

/// `client` contains the CLI client type and functions.
pub mod client;

/// `daemon` contains the CLI daemon type and functions.
pub mod daemon;
