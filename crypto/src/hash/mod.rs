//! # Hash
//!
//! `hash` is the module containing the types and functionalities used for hashing.

/// `digest` contains the digest type.
pub mod digest;
pub use self::digest::Digest;

/// `traits` contains the hashing traits.
pub mod traits;
pub use self::traits::*;

/// `blake2b512` contains the Blake2b512 hashing algorithm functions.
pub mod blake2b512;

/// `balloon` contains the Balloon hashing algorithm functions.
pub mod balloon;
