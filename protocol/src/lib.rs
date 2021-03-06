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

/// `client_server` contains the protocol client server type and functions.
pub mod client_server;

/// `consensus_server` contains the protocol consensus server type and functions.
pub mod consensus_server;

/// `miner_server` contains the protocol miner server type and functions.
pub mod miner_server;

/// `aliases` contains the main aliases of the crate.
pub mod aliases;

pub use crate::aliases::*;
