//! # Store
//!
//! `store` contains Alsacoin`s storage types and algorithms.

#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `traits` contains the storage traits.
pub mod traits;

/// `backend` contains the store backends.
pub mod backend;

/// `memory` contains the memory store type and functions.
pub mod memory;

/// `temporary` contains the temporary store type and functions.
pub mod temporary;

/// `persistent` contains the persistent store type and functions.
pub mod persistent;

/// `store` contains the store type and functions.
pub mod store;

/// `pool` contains the pool type and functions.
pub mod pool;

pub use crate::pool::PoolFactory;
pub use crate::store::StoreFactory;
