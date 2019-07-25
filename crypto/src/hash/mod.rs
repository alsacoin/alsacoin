//! # Hash
//!
//! `hash` is the module containing the types and functionalities used for hashing.

/// `digest` contains the digest type.
pub mod digest;
pub use self::digest::Digest;

/// `blake512` contains the Blake512 hashing algorithm functions.
pub mod blake512;
pub use self::blake512::Blake512Hasher;

/// `balloon` contains the Balloon hashing algorithm functions.
pub mod balloon;
pub use self::balloon::{BalloonHasher, BalloonParams};
