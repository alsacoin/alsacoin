//! # Signer
//!
//! `signer` is the module containing the output signer type and functions.

use crate::address::Address;
use crate::result::Result;
use crypto::random::Random;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;

/// `Signer` is a single signer of a `Transaction` `Output`.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Signer {
    pub address: Address,
    pub weight: u64,
}

impl Signer {
    /// `random` creates a new random `Signer`.
    pub fn random() -> Result<Signer> {
        let signer = Signer {
            address: Address::random()?,
            weight: Random::u64()?,
        };

        Ok(signer)
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