//! # Blake512
//!
//! `blake512` is the module containing the Blake2b512 hashing functions.

use crate::hash::Digest;
use crate::hash::CRH;
use digest::Digest as DigestT;

pub struct Blake512;

impl CRH for Blake512 {
    fn hash(&self, msg: &[u8]) -> Digest {
        let mut buf = [0u8; 64];

        for (i, v) in blake_hash::Blake512::digest(msg).iter().enumerate() {
            buf[i] = *v;
        }

        Digest::from_bytes(buf)
    }
}
