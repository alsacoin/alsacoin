//! # Signers
//!
//! `signers` is the module containing the account signers type and functions.

use crate::address::Address;
use crate::error::Error;
use crate::result::Result;
use crate::signer::Signer;
use crypto::ecc::ed25519::PublicKey;
use crypto::hash::Blake512Hasher;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use std::collections::BTreeMap;

/// `Signers` contains the signers of an `Account`, with their weight and threshold.
#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Signers {
    pub address: Address,
    pub signers: BTreeMap<PublicKey, Signer>,
    pub threshold: u64,
}

impl Signers {
    /// `new` creates a new `Signers`.
    pub fn new() -> Result<Signers> {
        let mut signers = Signers::default();
        signers.update_address()?;

        Ok(signers)
    }

    /// `calc_address` calculates the `Signers` address.
    pub fn calc_address(&self) -> Result<Address> {
        let mut clone = self.clone();
        clone.address = Address::default();

        let buf = clone.to_bytes()?;
        let digest = Blake512Hasher::hash(&buf);
        Ok(digest)
    }

    /// `update_address` updates the `Signers` address.
    pub fn update_address(&mut self) -> Result<()> {
        self.address = self.calc_address()?;

        Ok(())
    }

    /// `max_weight` returns the maximum weight in `Signers`.
    pub fn max_weight(&self) -> u64 {
        let mut max_weight = 0;

        for signer in self.signers.values() {
            if signer.weight > max_weight {
                max_weight = signer.weight;
            }
        }

        max_weight
    }

    /// `total_weight` returns the total weight in `Signers`.
    pub fn total_weight(&self) -> u64 {
        self.signers
            .values()
            .fold(0, |acc, signer| acc + signer.weight)
    }

    /// `set_threshold` sets the `Signers` threshold.
    pub fn set_threshold(&mut self, threshold: u64) -> Result<()> {
        self.threshold = threshold;

        self.update_address()
    }

    /// `lookup` looks up a signer in `Signers`.
    pub fn lookup(&self, public_key: &PublicKey) -> bool {
        self.signers.contains_key(public_key)
    }

    /// `get` gets a signer in `Signers`.
    pub fn get(&self, public_key: &PublicKey) -> Result<Signer> {
        if !self.lookup(public_key) {
            let err = Error::NotFound;
            return Err(err);
        }

        let signer = *self.signers.get(public_key).unwrap();
        Ok(signer)
    }

    /// `add` adds a signer in `Signers`.
    pub fn add(&mut self, signer: &Signer) -> Result<()> {
        if self.lookup(&signer.public_key) {
            let err = Error::AlreadyFound;
            return Err(err);
        }

        self.signers.insert(signer.public_key, signer.clone());

        self.update_address()
    }

    /// `update` updates a signer in `Signers`.
    pub fn update(&mut self, signer: &Signer) -> Result<()> {
        if !self.lookup(&signer.public_key) {
            let err = Error::NotFound;
            return Err(err);
        }

        if signer == &self.get(&signer.public_key)? {
            return Ok(());
        }

        self.signers.insert(signer.public_key, signer.clone());

        self.update_address()
    }

    /// `delete` deletes a signer in `Signers`.
    pub fn delete(&mut self, public_key: &PublicKey) -> Result<()> {
        if !self.lookup(public_key) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.signers.remove(public_key);

        self.update_address()
    }

    /// `validate` validates the `Signers`.
    pub fn validate(&self) -> Result<()> {
        if self.address != self.calc_address()? {
            let err = Error::InvalidAddress;
            return Err(err);
        }

        for (public_key, signer) in &self.signers {
            if public_key != &signer.public_key {
                let err = Error::InvalidPublicKey;
                return Err(err);
            }
        }

        if self.threshold > self.total_weight() {
            let err = Error::InvalidThreshold;
            return Err(err);
        }

        Ok(())
    }

    /// `to_bytes` converts the `Signers` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `Signerss`.
    pub fn from_bytes(b: &[u8]) -> Result<Signers> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `Signerss` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `Signerss`.
    pub fn from_json(s: &str) -> Result<Signers> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

#[test]
fn test_signers_ops() {
    let mut signers = Signers::new().unwrap();

    for _ in 0..10 {
        let signer = Signer::random().unwrap();

        let found = signers.lookup(&signer.public_key);
        assert!(!found);

        let res = signers.get(&signer.public_key);
        assert!(res.is_err());

        let res = signers.delete(&signer.public_key);
        assert!(res.is_err());

        let res = signers.add(&signer);
        assert!(res.is_ok());

        let found = signers.lookup(&signer.public_key);
        assert!(found);

        let res = signers.get(&signer.public_key);
        assert!(res.is_ok());

        let entry = res.unwrap();
        assert_eq!(signer, entry);

        let res = signers.delete(&signer.public_key);
        assert!(res.is_ok());

        let found = signers.lookup(&signer.public_key);
        assert!(!found);
    }
}

#[test]
fn test_signers_weight() {
    use crypto::random::Random;

    let mut signers = Signers::new().unwrap();
    let mut expected_max_weight = 0;
    let mut expected_total_weight = 0;

    for _ in 0..10 {
        let public_key = PublicKey::random().unwrap();
        let weight = Random::u64_range(1, 11).unwrap();
        let signer = Signer { public_key, weight };

        signers.add(&signer).unwrap();

        if signer.weight > expected_max_weight {
            expected_max_weight = signer.weight;
        }

        expected_total_weight += signer.weight;
    }

    let max_weight = signers.max_weight();
    let total_weight = signers.total_weight();
    assert_eq!(max_weight, expected_max_weight);
    assert_eq!(total_weight, expected_total_weight);
}

#[test]
fn test_signers_validate() {
    use crypto::random::Random;

    let mut signers = Signers::new().unwrap();

    let res = signers.validate();
    assert!(res.is_ok());

    for _ in 0..10 {
        let public_key = PublicKey::random().unwrap();
        let weight = Random::u64_range(1, 11).unwrap();
        let signer = Signer { public_key, weight };

        signers.add(&signer).unwrap();

        let res = signers.validate();
        assert!(res.is_ok());
    }

    let res = signers.validate();
    assert!(res.is_ok());

    signers.threshold = signers.total_weight() + 1;
    let res = signers.validate();
    assert!(res.is_err());

    signers.threshold = signers.total_weight();
    let res = signers.validate();
    assert!(res.is_err());

    let res = signers.set_threshold(signers.total_weight());
    assert!(res.is_ok());

    let res = signers.validate();
    assert!(res.is_ok());
}

#[test]
fn test_signers_serialize_bytes() {
    let signers_a = Signers::new().unwrap();

    let res = signers_a.to_bytes();
    assert!(res.is_ok());
    let cbor = res.unwrap();

    let res = Signers::from_bytes(&cbor);
    assert!(res.is_ok());
    let signers_b = res.unwrap();

    assert_eq!(signers_a, signers_b)
}

#[test]
fn test_signers_serialize_json() {
    let signers_a = Signers::new().unwrap();

    let res = signers_a.to_json();
    assert!(res.is_ok());
    let json = res.unwrap();

    let res = Signers::from_json(&json);
    assert!(res.is_ok());
    let signers_b = res.unwrap();

    assert_eq!(signers_a, signers_b)
}
