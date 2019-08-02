//! # Coinbase
//!
//! The `coinbase` module contains the `Coinbase` type and functions.

use crate::common::riemmann_zeta_2;
use crate::error::Error;
use crate::miner::Miner;
use crate::result::Result;
use crypto::hash::balloon::BalloonParams;
use crypto::hash::{Blake512Hasher, Digest};
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;

/// `COINBASE_BASE_AMOUNT` is the coinbase base amount.
pub const COINBASE_BASE_AMOUNT: u64 = 1_000_000_000;

/// `Coinbase` is the Alsacoin coinbase output type.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Coinbase {
    pub params: BalloonParams,
    pub distance: u64,
    pub difficulty: u64,
    pub nonce: u64,
    pub digest: Digest,
    pub amount: u64,
    pub checksum: Digest,
}

impl Coinbase {
    /// `new` creates a new unmined `Coinbase`.
    pub fn new(distance: u64, difficulty: u64) -> Result<Coinbase> {
        let mut coinbase = Coinbase::default();

        coinbase.distance = distance;
        coinbase.difficulty = difficulty;

        coinbase.update_amount()?;
        coinbase.update_checksum()?;

        Ok(coinbase)
    }

    /// `calc_amount` calculates the `Coinbase` amount given the transaction
    /// distance from the `Eve` transaction and mining difficulty.
    pub fn calc_amount(&self) -> Result<u64> {
        let distance = self.distance;
        let difficulty = self.difficulty;

        if (distance == 0) || (difficulty == 0) || (difficulty > 512) {
            let err = Error::OutOfBound;
            return Err(err);
        }

        let epoch = 1 + (distance as f64 / 1000f64) as u64;
        let res = ((COINBASE_BASE_AMOUNT as f64) * riemmann_zeta_2(epoch)?
            / riemmann_zeta_2(difficulty)?)
        .floor() as u64;
        Ok(res)
    }

    /// `set_amount` sets the `Coinbase` amount give the transaction
    /// distance from the `Eve` transaction and mining difficulty.
    pub fn update_amount(&mut self) -> Result<()> {
        self.amount = self.calc_amount()?;

        Ok(())
    }

    /// `calc_checksum` calculates the `Coinbase` checksum.
    pub fn calc_checksum(&self) -> Result<Digest> {
        let mut copy = *self;
        copy.checksum = Digest::default();

        let buf = copy.to_bytes()?;
        let checksum = Blake512Hasher::hash(&buf);
        Ok(checksum)
    }

    /// `update_checksum` updates the `Coinbase` checksum.
    pub fn update_checksum(&mut self) -> Result<()> {
        self.checksum = self.calc_checksum()?;
        Ok(())
    }

    /// `mining_message` returns the `Coinbase` mining message
    /// given an other provided binary message.
    pub fn mining_message(self, msg: &[u8]) -> Result<Vec<u8>> {
        let mut mmsg = Vec::new();

        mmsg.extend_from_slice(msg);

        let mut copy = self;
        copy.nonce = 0;
        copy.digest = Digest::default();
        copy.checksum = Digest::default();

        let buf = copy.to_bytes()?;

        mmsg.extend_from_slice(&buf);

        Ok(mmsg)
    }

    /// `calc_mining_proof` mines the `Coinbase` without
    /// updating it.
    pub fn calc_mining_proof(&self, msg: &[u8]) -> Result<(u64, Digest)> {
        let miner = Miner::new(self.params, self.difficulty)?;
        let mmsg = self.mining_message(msg)?;

        miner.mine_message(&mmsg)
    }

    /// `mine` mines the `Coinbase`.
    pub fn mine(&mut self, msg: &[u8]) -> Result<()> {
        let (nonce, digest) = self.calc_mining_proof(msg)?;

        self.nonce = nonce;
        self.digest = digest;

        self.update_checksum()
    }

    /// `validate` validates the unmined `Coinbase`.
    pub fn validate(&self) -> Result<()> {
        self.params.validate()?;

        if (self.distance == 0) || (self.difficulty == 0) || (self.difficulty > 512) {
            let err = Error::OutOfBound;
            return Err(err);
        }

        Ok(())
    }

    /// `validate_mining_proof` validates the `Coinbase` mining proof.
    pub fn validate_mining_proof(&self, msg: &[u8]) -> Result<()> {
        self.validate()?;

        let miner = Miner::new(self.params, self.difficulty)?;
        let mmsg = self.mining_message(msg)?;

        miner.verify_message_mining(&mmsg, self.nonce, self.digest)
    }

    /// `to_bytes` converts the `Coinbase` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `Coinbase`.
    pub fn from_bytes(b: &[u8]) -> Result<Coinbase> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `Coinbase` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `Coinbase`.
    pub fn from_json(s: &str) -> Result<Coinbase> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

impl Default for Coinbase {
    fn default() -> Coinbase {
        let mut coinbase = Coinbase {
            params: BalloonParams::default(),
            distance: 1,
            difficulty: 1,
            nonce: 0,
            digest: Digest::default(),
            amount: 0,
            checksum: Digest::default(),
        };

        coinbase.update_checksum().unwrap();

        coinbase
    }
}

#[test]
fn test_coinbase_new() {
    let hs = [1, 1000, 1_000_000];
    let ds = [1, 255, 512];

    let res = Coinbase::new(0, 1);
    assert!(res.is_err());

    let res = Coinbase::new(1, 0);
    assert!(res.is_err());

    for h in hs.iter() {
        for d in ds.iter() {
            let res = Coinbase::new(*h, *d);
            assert!(res.is_ok());
        }
    }
}

#[test]
fn test_coinbase_amount() {
    let hs = [1, 1000, 1_000_000];
    let ds = [1, 255, 512];
    let expected = [
        [COINBASE_BASE_AMOUNT, 609377028, 608649080],
        [1250000000, 761721286, 760811350],
        [1643935564, 1001776569, 1000579870],
    ];

    let mut coinbase = Coinbase::default();
    coinbase.distance = 0;
    coinbase.difficulty = 1;
    let res = coinbase.calc_amount();
    assert!(res.is_err());

    let mut coinbase = Coinbase::default();
    coinbase.distance = 1;
    coinbase.difficulty = 0;
    let res = coinbase.calc_amount();
    assert!(res.is_err());

    for (i, h) in hs.iter().enumerate() {
        for (j, d) in ds.iter().enumerate() {
            let res = Coinbase::new(*h, *d);
            assert!(res.is_ok());

            let mut coinbase = res.unwrap();
            let res = coinbase.calc_amount();
            assert!(res.is_ok());

            let amount = res.unwrap();
            let res = coinbase.update_amount();
            assert!(res.is_ok());

            assert_eq!(amount, coinbase.amount);
            assert_eq!(amount, expected[i][j])
        }
    }
}

#[test]
fn test_coinbase_validate() {
    use crypto::random::Random;

    let invalid_params = BalloonParams {
        s_cost: 0,
        t_cost: 0,
        delta: 0,
    };

    let invalid_distance = 0;
    let invalid_difficulty = 513;

    for _ in 0..10 {
        let valid_params = BalloonParams {
            s_cost: Random::u32_range(1, 3).unwrap(),
            t_cost: Random::u32_range(1, 3).unwrap(),
            delta: Random::u32_range(3, 6).unwrap(),
        };
        let valid_distance = Random::u64_range(1, 10).unwrap();
        let valid_difficulty = Random::u64_range(1, 10).unwrap();

        let mut coinbase = Coinbase::default();
        let res = coinbase.validate();
        assert!(res.is_ok());

        coinbase.params = valid_params;
        coinbase.distance = valid_distance;
        coinbase.difficulty = valid_difficulty;
        let res = coinbase.validate();
        assert!(res.is_ok());

        coinbase.params = invalid_params;
        let res = coinbase.validate();
        assert!(res.is_err());
        coinbase.params = valid_params;

        coinbase.distance = invalid_distance;
        let res = coinbase.validate();
        assert!(res.is_err());
        coinbase.distance = valid_distance;

        coinbase.difficulty = invalid_difficulty;
        let res = coinbase.validate();
        assert!(res.is_err());
    }
}

#[test]
fn test_coinbase_mine() {
    use crypto::random::Random;

    for _ in 0..10 {
        let distance = Random::u64_range(1, 3).unwrap();
        let difficulty = Random::u64_range(1, 3).unwrap();
        let msg_len = 1000;
        let msg = Random::bytes(msg_len).unwrap();

        let mut coinbase = Coinbase::new(distance, difficulty).unwrap();
        let res = coinbase.validate_mining_proof(&msg);
        assert!(res.is_err());

        let res = coinbase.calc_mining_proof(&msg);
        assert!(res.is_ok());

        let (nonce, digest) = res.unwrap();
        let res = coinbase.mine(&msg);
        assert!(res.is_ok());

        assert_eq!(coinbase.nonce, nonce);
        assert_eq!(coinbase.digest, digest);

        let res = coinbase.validate_mining_proof(&msg);
        assert!(res.is_ok());
    }
}

#[test]
fn test_coinbase_serialize_bytes() {
    use crypto::random::Random;

    for _ in 0..10 {
        let distance = Random::u64_range(1, 10).unwrap();
        let difficulty = Random::u64_range(1, 10).unwrap();
        let coinbase_a = Coinbase::new(difficulty, distance).unwrap();

        let res = coinbase_a.to_bytes();
        assert!(res.is_ok());
        let cbor = res.unwrap();

        let res = Coinbase::from_bytes(&cbor);
        assert!(res.is_ok());
        let coinbase_b = res.unwrap();

        assert_eq!(coinbase_a, coinbase_b)
    }
}

#[test]
fn test_coinbase_serialize_json() {
    use crypto::random::Random;

    for _ in 0..10 {
        let distance = Random::u64_range(1, 10).unwrap();
        let difficulty = Random::u64_range(1, 10).unwrap();
        let coinbase_a = Coinbase::new(difficulty, distance).unwrap();

        let res = coinbase_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = Coinbase::from_json(&json);
        assert!(res.is_ok());
        let coinbase_b = res.unwrap();

        assert_eq!(coinbase_a, coinbase_b)
    }
}
