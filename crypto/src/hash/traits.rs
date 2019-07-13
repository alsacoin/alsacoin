//! # Traits
//!
//! `traits` contains the hashing traits used in the crate.

use crate::hash::Digest;

/// `CRH` is the trait used by types implementing Collision Resistant
/// Hashing (CRH).
pub trait CRH {
    /// `hash` returns the hash of a binary message.
    fn hash(&self, msg: &[u8]) -> Digest;
}
