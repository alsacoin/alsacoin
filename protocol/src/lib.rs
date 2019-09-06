//! # Consensus
//!
//! `consensus` contains Alsacoin`s consensus types and functions.

#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `network` contains the protocol network functions.
pub mod network;

/// `state` contains the protocol state type and functions.
pub mod state;

/// `client` contains the protocol client type and functions.
pub mod client;

/// `server` contains the protocol main server type and functions.
pub mod server;

/// `miner` contains the protocol miner type and functions.
pub mod miner;
