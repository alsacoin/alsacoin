//! # Crypto
//!
//! The `crypto` crate contains the cryptographic types and functionalities
//! used in Alsacoin.

#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `random` is the module containing the random functions.
pub mod random;

/// `hash` is the module containing the hashing functions.
pub mod hash;
