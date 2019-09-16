//! # Network
//!
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

/// `backend` contains the networking backends.
pub mod backend;

/// `network` contains the network type and functions.
pub mod network;

pub use crate::network::NetworkFactory;
