//! # Blake512
//!
//! `blake512` is the module containing the Blake2b512 hashing functions.

use crate::hash;
use crate::hash::CRH;
use digest::Digest;

pub struct Blake512Hasher;

impl CRH for Blake512Hasher {
    fn hash(&self, msg: &[u8]) -> hash::Digest {
        let mut buf = [0u8; 64];

        for (i, v) in blake_hash::Blake512::digest(msg).iter().enumerate() {
            buf[i] = *v;
        }

        hash::Digest::from_bytes(buf)
    }
}
