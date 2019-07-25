//! # Mining
//!
//! `mining` is the crate that contains Alsacoin's mining types and functions.

#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `common` contains functionalities used throughout the crate.
mod common;

/// `difficulty` contains the difficulty functions.
pub mod difficulty;

/// `coinbase` contains the coinbase generation functions.
pub mod coinbase;

/// `miner` contains the mining types and functions.
pub mod miner;
