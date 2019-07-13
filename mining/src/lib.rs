//! # Mining
//!
//! `mining` is the crate that contains Alsacoin's mining types and functions.

/// `common` contains functionalities used throughout the crate.
mod common;

/// `difficulty` contains the difficulty functions.
pub mod difficulty;

/// `target` contains the target functions.
pub mod target;

/// `coinbase` contains the coinbase generation functions.
pub mod coinbase;
