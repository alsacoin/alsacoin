//! # Wallet
//!
//! `wallet` contains the `Wallet` type and functions.

use crate::account::Account;
use crate::error::Error;
use crate::result::Result;
use crypto::ecc::ed25519::{KeyPair, PublicKey, SecretKey, Signature};
use crypto::hash::{Blake512Hasher, Digest};
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;

/// `Wallet` is an account history related to a same `Address`
/// and its related `KeyPair`.
#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Wallet {
    pub public_key: PublicKey,
    pub history: Vec<Account>,
    pub signature: Signature,
    pub checksum: Digest,
}

impl Wallet {
    /// `new` creates a new `Wallet`.
    pub fn new(secret_key: &SecretKey) -> Result<Wallet> {
        let mut wallet = Wallet::default();

        wallet.public_key = secret_key.to_public();
        wallet.sign(secret_key)?;
        wallet.update_checksum()?;
        wallet.validate()?;

        Ok(wallet)
    }

    /// `calc_sign` calculates the `Wallet` signature.
    pub fn calc_sign(&self, secret_key: &SecretKey) -> Result<Signature> {
        let kp = KeyPair::from_secret(secret_key)?;

        if kp.public_key != self.public_key {
            let err = Error::InvalidPublicKey;
            return Err(err);
        }

        let mut clone = self.clone();
        clone.signature = Signature::default();
        clone.checksum = Digest::default();

        let buf = clone.to_bytes()?;
        kp.sign(&buf).map_err(|e| e.into())
    }

    /// `calc_checksum` calculates the `Wallet` checksum.
    pub fn calc_checksum(&self) -> Result<Digest> {
        let mut clone = self.clone();
        clone.checksum = Digest::default();

        let buf = clone.to_bytes()?;
        let checksum = Blake512Hasher::hash(&buf);
        Ok(checksum)
    }

    /// `sign` signs the `Wallet` and update its id.
    pub fn sign(&mut self, secret_key: &SecretKey) -> Result<()> {
        self.signature = self.calc_sign(secret_key)?;

        Ok(())
    }

    /// `update_checksum` updates the `Wallet` checksum.
    pub fn update_checksum(&mut self) -> Result<()> {
        self.checksum = self.calc_checksum()?;

        Ok(())
    }

    /// `validate` validates the `Wallet`.
    pub fn validate(&self) -> Result<()> {
        for account in &self.history {
            account.validate()?;
        }

        if self.checksum != self.calc_checksum()? {
            let err = Error::InvalidChecksum;
            return Err(err);
        }

        Ok(())
    }

    /// `validate_signature` validates the `Wallet` signature.
    pub fn validate_signature(&self, secret_key: &SecretKey) -> Result<()> {
        self.validate()?;

        if self.signature != self.calc_sign(secret_key)? {
            let err = Error::InvalidSignature;
            return Err(err);
        }

        Ok(())
    }

    /// `latest_account` returns the most recent `Account`.
    pub fn latest_account(&self) -> Option<Account> {
        self.history.last().copied()
    }

    /// `update_history` updates the `Wallet`'s `Account` history.
    pub fn update_history(&mut self, secret_key: &SecretKey, value: u64) -> Result<()> {
        self.validate()?;

        let new_account = if let Some(account) = self.latest_account() {
            account.update(value)?
        } else {
            Account::new(self.public_key, value)?
        };

        self.history.push(new_account);

        self.sign(secret_key)?;
        self.update_checksum()?;

        Ok(())
    }

    /// `to_bytes` converts the `Wallet` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `Wallet`.
    pub fn from_bytes(b: &[u8]) -> Result<Wallet> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `Wallet` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `Wallet`.
    pub fn from_json(s: &str) -> Result<Wallet> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

#[test]
fn test_wallet_new() {
    let secret_key = SecretKey::random().unwrap();
    let res = Wallet::new(&secret_key);
    assert!(res.is_ok());

    let wallet = res.unwrap();

    let res = wallet.validate();
    assert!(res.is_ok());

    let res = wallet.validate_signature(&secret_key);
    assert!(res.is_ok());
}

#[test]
fn test_wallet_update_history() {
    use crypto::random::Random;

    let secret_key = SecretKey::random().unwrap();
    let res = Wallet::new(&secret_key);
    assert!(res.is_ok());

    let mut wallet = res.unwrap();

    for _ in 0..10 {
        let value = Random::u64().unwrap();
        let res = wallet.update_history(&secret_key, value);
        assert!(res.is_ok());

        let res = wallet.validate();
        assert!(res.is_ok());

        let res = wallet.validate_signature(&secret_key);
        assert!(res.is_ok());
    }
}

#[test]
fn test_wallet_serialize_bytes() {
    for _ in 0..10 {
        let secret_key = SecretKey::random().unwrap();
        let wallet_a = Wallet::new(&secret_key).unwrap();

        let res = wallet_a.to_bytes();
        assert!(res.is_ok());
        let cbor = res.unwrap();

        let res = Wallet::from_bytes(&cbor);
        assert!(res.is_ok());
        let wallet_b = res.unwrap();

        assert_eq!(wallet_a, wallet_b)
    }
}

#[test]
fn test_wallet_serialize_json() {
    for _ in 0..10 {
        let secret_key = SecretKey::random().unwrap();
        let wallet_a = Wallet::new(&secret_key).unwrap();

        let res = wallet_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = Wallet::from_json(&json);
        assert!(res.is_ok());
        let wallet_b = res.unwrap();

        assert_eq!(wallet_a, wallet_b)
    }
}
