//! # Config
//!
//! `config` contains Alsacoin`s config types and functions.

#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `store_config` contains the store configurations type and functions.
pub mod store_config;

/// `pool_config` contains the pool configurations type and functions.
pub mod pool_config;

/// `network_config` contains the network configurations type and functions.
pub mod network_config;

/// `consensus_config` contains the consensus configurations type and functions.
pub mod consensus_config;

/// `config` contains the  configurations type and functions.
pub mod config;
