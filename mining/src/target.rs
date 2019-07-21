//! # Target
//!
//! The `target` module contains the mining target functions.

use crate::error::Error;
use crate::result::Result;
use std::convert::From;
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone)]
pub struct Target([u8; 64]);

impl Target {
    /// `new` creates a new `Target` from a given number of bits. Alias of `from_bits`.
    pub fn new(b: u64) -> Result<Target> {
        Target::from_bits(b)
    }

    /// `from_bytes` creates a new `Target` from an array of bytes.
    pub fn from_bytes(t: [u8; 64]) -> Target {
        Target(t)
    }

    /// `to_bytes` converts the `Target` into an array of bytes.
    pub fn to_bytes(&self) -> [u8; 64] {
        self.0
    }

    /// `from_bits` generates the target bytes from a given number of bits.
    pub fn from_bits(b: u64) -> Result<Target> {
        if b > 512 {
            let msg = Error::OutOfBound;
            return Err(msg);
        }

        let mut t = [255u8; 64];
        let zb = (b / 8) as usize;

        #[allow(clippy::needless_range_loop)]
        for i in 0..zb {
            t[i] = 0;
        }

        let rem = b % 8;
        if rem != 0 {
            t[zb] >>= rem;
        }

        Ok(Target(t))
    }

    /// `to_bits` returns the target leading 0 bits.
    pub fn to_bits(&self) -> u64 {
        let mut bits = 0;

        for b in self.0.iter() {
            if *b == 0 {
                bits += 8;
            } else {
                bits += u64::from(b.leading_zeros());
                break;
            }
        }

        bits
    }
}

impl Index<usize> for Target {
    type Output = u8;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0.index(idx)
    }
}

impl IndexMut<usize> for Target {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        self.0.index_mut(idx)
    }
}

impl From<[u8; 64]> for Target {
    fn from(t: [u8; 64]) -> Target {
        Target(t)
    }
}

#[test]
fn test_target() {
    let bits = [0, 1, 10, 100];

    let res = Target::new(513);
    assert!(res.is_err());

    let res = Target::new(1000);
    assert!(res.is_err());

    for b in bits.iter() {
        let res = Target::new(*b);
        assert!(res.is_ok());

        let t = res.unwrap();

        let br = t.to_bits();
        assert_eq!(*b, br);
    }
}
