//! # Mining
//!
//! `mining` is the crate that contains Alsacoin's mining types and functions.

#[macro_use]
extern crate failure;

#[macro_use]
extern crate enum_display_derive;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `common` contains functionalities used throughout the crate.
mod common;

/// `difficulty` contains the difficulty functions.
pub mod difficulty;

/// `target` contains the target functions.
pub mod target;

/// `coinbase` contains the coinbase generation functions.
pub mod coinbase;
