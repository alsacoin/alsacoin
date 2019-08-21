//! `network` contains Alsacoin`s networking types and functions.

#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `message` contains the networking message type and functions.
pub mod message;

/// `traits` contains the networking traits used in the crate.
pub mod traits;

/// `node` contains the networking nodes.
pub mod node;
