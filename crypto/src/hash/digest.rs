//! # Digest
//!
//! `digest` is the module containing the `Digest` type returned by hashing
//! algorithms.

use std::convert::From;
use std::ops::{Index, IndexMut};

/// `Digest` is the type returned by hashing algorithms.
#[derive(Copy, Clone)]
pub struct Digest([u8; 64]);

impl Digest {
    /// `new` creates a new `Digest` from an array of bytes. Alias of `from_bytes`.
    pub fn new(d: [u8; 64]) -> Digest {
        Digest(d)
    }

    /// `from_bytes` creates a new `Digest` from an array of bytes.
    pub fn from_bytes(d: [u8; 64]) -> Digest {
        Digest(d)
    }

    /// `to_bytes` converts the `Digest` into an array of bytes.
    pub fn to_bytes(&self) -> [u8; 64] {
        self.0
    }
}

impl Default for Digest {
    fn default() -> Digest {
        Digest([0u8; 64])
    }
}

impl Index<usize> for Digest {
    type Output = u8;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0.index(idx)
    }
}

impl IndexMut<usize> for Digest {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        self.0.index_mut(idx)
    }
}

impl From<[u8; 64]> for Digest {
    fn from(t: [u8; 64]) -> Digest {
        Digest(t)
    }
}
