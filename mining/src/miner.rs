//! # Miner
//!
//! The `miner` module contains the mining types and functions used in Alsacoin.

use crate::error::Error;
use crate::result::Result;
use crypto::hash::Blake512Hasher;
use crypto::hash::Digest;
use crypto::hash::{BalloonHasher, BalloonParams};
use std::mem::transmute;

/// `Miner` is the type used for mining.
pub struct Miner {
    params: BalloonParams,
    difficulty: u64,
}

impl Miner {
    /// `new` creates a new `Miner`.
    pub fn new(params: BalloonParams, difficulty: u64) -> Result<Miner> {
        params.validate()?;

        if difficulty > 512 {
            let err = Error::OutOfBound;
            return Err(err);
        }

        let miner = Miner { params, difficulty };
        Ok(miner)
    }

    /// `validate` validates the `Miner`.
    pub fn validate(&self) -> Result<()> {
        self.params.validate()?;

        if self.difficulty > 512 {
            let err = Error::OutOfBound;
            return Err(err);
        }

        Ok(())
    }

    /// `hash_message` returns the hash of a binary message.
    pub fn hash_message(&self, msg: &[u8]) -> Result<Digest> {
        self.validate()?;

        let salt = Blake512Hasher::hash(msg);
        let params = self.params;
        let hasher = BalloonHasher { salt, params };

        hasher.hash(msg).map_err(|e| e.into())
    }

    /// `nonced_message` returns a binary message by appending a nonce
    /// to an other binary message.
    pub fn nonced_message(nonce: u64, msg: &[u8]) -> Vec<u8> {
        let mut nmsg = Vec::new();

        let head: [u8; 8] = unsafe { transmute::<u64, [u8; 8]>(nonce) };

        nmsg.extend_from_slice(&head);
        nmsg.extend_from_slice(msg);
        nmsg
    }

    /// `mine_message` mines a binary message.
    pub fn mine_message(&self, msg: &[u8]) -> Result<(u64, Digest)> {
        let mut nonce = 0u64;

        while nonce <= u64::max_value() {
            let nmsg = Miner::nonced_message(nonce, msg);
            let hash = self.hash_message(&nmsg)?;
            let bits = hash.leading_zeros();

            if bits >= self.difficulty {
                return Ok((nonce, hash));
            }

            nonce += 1;
        }

        let err = Error::NotFound;
        Err(err)
    }
}

#[test]
fn test_mine_message() {
    use crypto::random::Random;

    let params = BalloonParams::default();
    let msg_len = 1000;
    let msg = Random::bytes(msg_len).unwrap();
    let diffs = [0, 1, 2, 3];

    for diff in diffs.iter() {
        let res = Miner::new(params, *diff);
        assert!(res.is_ok());

        let miner = res.unwrap();
        let res = miner.mine_message(&msg);
        assert!(res.is_ok());

        let (_, digest) = res.unwrap();
        let bits = digest.leading_zeros();
        assert!(bits >= *diff)
    }
}
