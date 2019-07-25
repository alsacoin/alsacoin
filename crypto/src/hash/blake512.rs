//! # Blake512
//!
//! `blake512` is the module containing the Blake2b512 hashing functions.

use crate::hash;
use digest::Digest;

/// `Blake512Hasher` is the type implementing Blake2b512 hashing.
pub struct Blake512Hasher;

impl Blake512Hasher {
    pub fn hash(msg: &[u8]) -> hash::Digest {
        let mut buf = [0u8; 64];

        for (i, v) in blake_hash::Blake512::digest(msg).iter().enumerate() {
            buf[i] = *v;
        }

        hash::Digest::from_bytes(buf)
    }
}
