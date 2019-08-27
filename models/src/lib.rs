//! `model` contains Alsacoin`s model types and functions.

#![feature(async_await)]

#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `utils` contains various utilities used in the crate.
pub mod utils;

/// `traits` contains traits used in the crate.
pub mod traits;

/// `timestamp` contains the timestamping types and functions.
pub mod timestamp;

/// `stage` contains the stage type and functions.
pub mod stage;

/// `version` contains the version type and functions.
pub mod version;

/// `node` contains the node type and functions.
pub mod node;

/// `address` contains the address type and functions.
pub mod address;

/// `signer` contains the signer type and functions.
pub mod signer;

/// `signers` contains the signers type and functions.
pub mod signers;

/// `account` contains the account type and functions.
pub mod account;

/// `input` contains the input type and functions.
pub mod input;

/// `ouput` contains the output type and functions.
pub mod output;

/// `coinbase` contains the coinbase type and functions.
pub mod coinbase;

/// `transaction` contains the transaction type and functions.
pub mod transaction;

/// `consensus_params` contains the consensus parameters type and functions.
pub mod consensus_params;

/// `conflict_set` contains the conflict set type and functions.
pub mod conflict_set;

/// `consensus_state` contains the consensus state type and functions.
pub mod consensus_state;

/// `consensus_message` contains the consensus message type and functions.
pub mod consensus_message;
