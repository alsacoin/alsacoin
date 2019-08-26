#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `message` contains the consensus message type and functions.
pub mod message;

/// `consensus` contains the Avalanche Consensus type and functions.
pub mod consensus;
