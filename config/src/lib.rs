//! # Config
//!
//! `config` contains Alsacoin`s configuration types and functions.

#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `store_config` contains the store configuration type and functions.
pub mod store_config;

/// `pool_config` contains the pool configuration type and functions.
pub mod pool_config;

/// `network_config` contains the network configuration type and functions.
pub mod network_config;

/// `log_config` contains the logging configuration
pub mod log_config;

/// `consensus_config` contains the consensus configuration type and functions.
pub mod consensus_config;

/// `config` contains the  configuration type and functions.
pub mod config;
