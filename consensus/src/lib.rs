//! # Consensus
//!
//! `consensus` contains Alsacoin`s consensus types and functions.

#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `protocol` contains the Avalanche Consensus Protocol type and functions.
pub mod protocol;
