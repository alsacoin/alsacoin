//! `store` contains Alsacoin`s storage types and algorithms.

#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `traits` contains the storage traits.
pub mod traits;

/// `memory` contains the memory store type and functions.
pub mod memory;

/// `persistent` contains the persistent store type and functions.
pub mod persistent;
