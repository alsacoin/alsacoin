//! # Config
//!
//! `config` contains Alsacoin`s configuration types and functions.

#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `store` contains the store configuration type and functions.
pub mod store;

/// `pool` contains the pool configuration type and functions.
pub mod pool;

/// `network` contains the network configuration type and functions.
pub mod network;

/// `log` contains the logging configuration
pub mod log;

/// `consensus` contains the consensus configuration type and functions.
pub mod consensus;

/// `config` contains the  configuration type and functions.
pub mod config;
