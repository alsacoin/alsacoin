//! `store` contains Alsacoin`s storage types and algorithms.

#[macro_use]
extern crate failure;

#[macro_use]
extern crate enum_display_derive;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `traits` contains the storage traits.
pub mod traits;

pub use traits::*;
