//! # Coinbase
//!
//! The `coinbase` module contains the `Coinbase` type and functions.

use crate::address::Address;
use crate::error::Error;
use crate::result::Result;
use crypto::hash::balloon::BalloonParams;
use crypto::hash::Digest;
use mining::common::riemmann_zeta_2;
use mining::miner::Miner;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;

/// `Coinbase` is the Alsacoin coinbase output type.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
pub struct Coinbase {
    pub address: Address,
    pub distance: u64,
    pub difficulty: u64,
    pub custom_digest: Digest,
    pub amount: u64,
    pub params: BalloonParams,
    pub nonce: u64,
    pub digest: Digest,
}

impl Coinbase {
    /// `BASE_AMOUNT` is the coinbase base amount.
    pub const BASE_AMOUNT: u64 = 1_000_000_000;

    /// `new` creates a new unmined `Coinbase`.
    pub fn new(address: &Address, distance: u64, difficulty: u64) -> Result<Coinbase> {
        if distance == 0 && difficulty != 0 {
            let err = Error::InvalidDistance;
            return Err(err);
        }

        if difficulty == 0 && distance != 0 {
            let err = Error::InvalidDifficulty;
            return Err(err);
        }

        let mut coinbase = Coinbase::default();
        coinbase.address = address.to_owned();
        coinbase.distance = distance;
        coinbase.difficulty = difficulty;
        coinbase.update_amount()?;

        Ok(coinbase)
    }

    /// `new_eve` creates a new unmined eve `Coinbase`.
    pub fn new_eve(address: &Address) -> Result<Coinbase> {
        Coinbase::new(address, 0, 0)
    }

    /// `is_eve` returns if the `Coinbase` is a eve `Coinbase`.
    pub fn is_eve(&self) -> Result<bool> {
        self.validate()?;

        let res = self.distance == 0 && self.difficulty == 0;

        Ok(res)
    }

    /// `clear` clears the `Coinbase` of the mining proof.
    pub fn clear(&mut self) {
        self.nonce = 0;
        self.digest = Digest::default();
    }

    /// `calc_amount` calculates the `Coinbase` amount given the transaction
    /// distance from the `Eve` transaction and mining difficulty.
    pub fn calc_amount(&self) -> Result<u64> {
        let distance = self.distance;
        let difficulty = self.difficulty;

        if ((distance == 0) ^ (difficulty == 0)) || (difficulty > 512) {
            let err = Error::OutOfBound;
            return Err(err);
        }

        if distance == 0 && difficulty == 0 {
            return Ok(Coinbase::BASE_AMOUNT);
        }

        let epoch = 1 + (distance as f64 / 1000f64) as u64;
        let res = ((Coinbase::BASE_AMOUNT as f64) * riemmann_zeta_2(epoch)?
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

    /// `mining_message` returns the `Coinbase` mining message
    /// given an other provided binary message.
    pub fn mining_message(self, msg: &[u8]) -> Result<Vec<u8>> {
        let mut mmsg = Vec::new();

        mmsg.extend_from_slice(msg);

        let mut copy = self;
        copy.nonce = 0;
        copy.digest = Digest::default();

        let buf = copy.to_bytes()?;

        mmsg.extend_from_slice(&buf);

        Ok(mmsg)
    }

    /// `calc_mining_proof` mines the `Coinbase` without
    /// updating it.
    pub fn calc_mining_proof(&self, msg: &[u8]) -> Result<(u64, Digest)> {
        let miner = Miner::new(self.params, self.difficulty)?;
        let mmsg = self.mining_message(msg)?;

        miner.mine_message(&mmsg).map_err(|e| e.into())
    }

    /// `mine` mines the `Coinbase`.
    pub fn mine(&mut self, msg: &[u8]) -> Result<()> {
        let (nonce, digest) = self.calc_mining_proof(msg)?;

        self.nonce = nonce;
        self.digest = digest;

        Ok(())
    }

    /// `validate` validates the unmined `Coinbase`.
    pub fn validate(&self) -> Result<()> {
        self.params.validate()?;

        if ((self.distance == 0) ^ (self.difficulty == 0)) || (self.difficulty > 512) {
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

        miner
            .verify_message_mining(&mmsg, self.nonce, self.digest)
            .map_err(|e| e.into())
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
        Coinbase {
            address: Address::default(),
            params: BalloonParams::default(),
            distance: 1,
            difficulty: 1,
            custom_digest: Digest::default(),
            nonce: 0,
            digest: Digest::default(),
            amount: 0,
        }
    }
}

#[test]
fn test_coinbase_new() {
    let hs = [1, 1000, 1_000_000];
    let ds = [1, 255, 512];

    let address = Address::random().unwrap();
    let res = Coinbase::new(&address, 0, 1);
    assert!(res.is_err());

    let res = Coinbase::new(&address, 1, 0);
    assert!(res.is_err());

    for h in hs.iter() {
        for d in ds.iter() {
            let res = Coinbase::new(&address, *h, *d);
            assert!(res.is_ok());
        }
    }
}

#[test]
fn test_coinbase_new_eve() {
    let address = Address::random().unwrap();

    let res = Coinbase::new_eve(&address);
    assert!(res.is_ok());

    let mut eve_coinbase = res.unwrap();

    let res = eve_coinbase.validate();
    assert!(res.is_ok());

    eve_coinbase.distance = 1;

    let res = eve_coinbase.is_eve();
    assert!(res.is_err());

    eve_coinbase.distance = 0;

    let res = eve_coinbase.is_eve();
    assert!(res.is_ok());
    assert!(res.unwrap());

    let hs = [1, 1000, 1_000_000];
    let ds = [1, 255, 512];

    for h in hs.iter() {
        for d in ds.iter() {
            let coinbase = Coinbase::new(&address, *h, *d).unwrap();
            let res = coinbase.is_eve();
            assert!(res.is_ok());
            assert!(!res.unwrap());
        }
    }
}

#[test]
fn test_coinbase_amount() {
    let hs = [1, 1000, 1_000_000];
    let ds = [1, 255, 512];
    let expected = [
        [Coinbase::BASE_AMOUNT, 609377028, 608649080],
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
            let address = Address::random().unwrap();
            let res = Coinbase::new(&address, *h, *d);
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
        let address = Address::random().unwrap();
        let distance = Random::u64_range(1, 3).unwrap();
        let difficulty = Random::u64_range(1, 3).unwrap();
        let msg_len = 1000;
        let msg = Random::bytes(msg_len).unwrap();

        let mut coinbase = Coinbase::new(&address, distance, difficulty).unwrap();
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

        coinbase.clear();
        assert_eq!(coinbase.nonce, 0);
        assert_eq!(coinbase.digest, Digest::default());
    }
}

#[test]
fn test_coinbase_serialize_bytes() {
    use crypto::random::Random;

    for _ in 0..10 {
        let address = Address::random().unwrap();
        let distance = Random::u64_range(1, 10).unwrap();
        let difficulty = Random::u64_range(1, 10).unwrap();
        let coinbase_a = Coinbase::new(&address, difficulty, distance).unwrap();

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
        let address = Address::random().unwrap();
        let distance = Random::u64_range(1, 10).unwrap();
        let difficulty = Random::u64_range(1, 10).unwrap();
        let coinbase_a = Coinbase::new(&address, difficulty, distance).unwrap();

        let res = coinbase_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = Coinbase::from_json(&json);
        assert!(res.is_ok());
        let coinbase_b = res.unwrap();

        assert_eq!(coinbase_a, coinbase_b)
    }
}
