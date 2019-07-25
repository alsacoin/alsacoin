//! # Digest
//!
//! `digest` is the module containing the `Digest` type returned by hashing
//! algorithms.

use crate::error::Error;
use crate::result::Result;
use std::cmp::{Eq, PartialEq};
use std::convert::From;
use std::ops::{Index, IndexMut};
use subtle::ConstantTimeEq;

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

    /// `leading_zeros` returns the `Digest` leading zeros.
    pub fn leading_zeros(&self) -> u64 {
        let mut zeros = 0;

        for b in self.0.iter() {
            if *b == 0 {
                zeros += 8;
            } else {
                zeros += u64::from(b.leading_zeros());
                break;
            }
        }

        zeros
    }

    /// `from_leading_zeros` creates a `Digest` with a specific
    /// number of leading zeros.
    pub fn from_leading_zeros(zeros: u64) -> Result<Digest> {
        if zeros > 512 {
            let msg = Error::OutOfBound;
            return Err(msg);
        }

        let mut ds = [255u8; 64];
        let zb = (zeros / 8) as usize;

        #[allow(clippy::needless_range_loop)]
        for i in 0..zb {
            ds[i] = 0;
        }

        let rem = zeros % 8;
        if rem != 0 {
            ds[zb] >>= rem;
        }

        let d = Digest::from_bytes(ds);
        Ok(d)
    }
}

impl Default for Digest {
    fn default() -> Digest {
        Digest([0u8; 64])
    }
}

impl PartialEq for Digest {
    fn eq(&self, other: &Digest) -> bool {
        (&self.to_bytes()).ct_eq(&other.to_bytes()).unwrap_u8() == 1u8
    }
}

impl Eq for Digest {}

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
