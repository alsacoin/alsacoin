//! # Signer
//!
//! `signer` is the module containing the account signer type and functions.

use crate::result::Result;
use crate::wallet::Wallet;
use crypto::ecc::ed25519::PublicKey;
use crypto::random::Random;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;

/// `Signer` is a single signer of an `Account`.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct Signer {
    pub public_key: PublicKey,
    pub weight: u64,
}

impl Signer {
    /// `random` creates a new random `Signer`.
    pub fn random() -> Result<Signer> {
        let signer = Signer {
            public_key: PublicKey::random()?,
            weight: Random::u64()?,
        };

        Ok(signer)
    }

    /// `from_wallet` creates a `Signer` from a `Wallet`.
    pub fn from_wallet(wallet: &Wallet, weight: u64) -> Result<Signer> {
        wallet.to_signer(weight)
    }

    /// `to_bytes` converts the `Signer` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `Signer`.
    pub fn from_bytes(b: &[u8]) -> Result<Signer> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `Signer` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `Signer`.
    pub fn from_json(s: &str) -> Result<Signer> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

#[test]
fn test_signer_from_wallet() {
    use crate::stage::Stage;
    use crypto::ecc::ed25519::SecretKey;

    let stage = Stage::default();
    let weight = 10;
    let mut wallet = Wallet::new(stage).unwrap();
    let valid_secret = wallet.secret_key.clone();

    let res = Signer::from_wallet(&wallet, weight);
    assert!(res.is_ok());

    while wallet.secret_key == valid_secret {
        wallet.secret_key = SecretKey::random().unwrap().to_vec();
    }

    let res = Signer::from_wallet(&wallet, weight);
    assert!(res.is_err());
}

#[test]
fn test_signer_serialize_bytes() {
    for _ in 0..10 {
        let signer_a = Signer::random().unwrap();

        let res = signer_a.to_bytes();
        assert!(res.is_ok());
        let cbor = res.unwrap();

        let res = Signer::from_bytes(&cbor);
        assert!(res.is_ok());
        let signer_b = res.unwrap();

        assert_eq!(signer_a, signer_b)
    }
}

#[test]
fn test_signer_serialize_json() {
    for _ in 0..10 {
        let signer_a = Signer::random().unwrap();

        let res = signer_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = Signer::from_json(&json);
        assert!(res.is_ok());
        let signer_b = res.unwrap();

        assert_eq!(signer_a, signer_b)
    }
}
