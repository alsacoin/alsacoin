//! # Output
//!
//! `output` contains the `Output` type and functions.

use crate::address::Address;
use crate::result::Result;
use crypto::hash::Digest;
use crypto::random::Random;
use serde::{Deserialize, Serialize};

/// `Output` is an output in an Alsacoin `Transaction`.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct Output {
    pub address: Address,
    pub amount: u64,
    pub digest: Option<Digest>,
}

impl Output {
    /// `random` creates a random `Output`.
    pub fn random() -> Result<Output> {
        let output = Output {
            address: Address::random()?,
            amount: Random::u64()?,
            digest: None,
        };

        Ok(output)
    }

    /// `to_bytes` converts the `Output` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `Output`.
    pub fn from_bytes(b: &[u8]) -> Result<Output> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `Output` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `Output`.
    pub fn from_json(s: &str) -> Result<Output> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

#[test]
fn test_output_serialize_bytes() {
    for _ in 0..10 {
        let output_a = Output::random().unwrap();

        let res = output_a.to_bytes();
        assert!(res.is_ok());
        let cbor = res.unwrap();

        let res = Output::from_bytes(&cbor);
        assert!(res.is_ok());
        let output_b = res.unwrap();

        assert_eq!(output_a, output_b)
    }
}

#[test]
fn test_output_serialize_json() {
    for _ in 0..10 {
        let output_a = Output::random().unwrap();

        let res = output_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = Output::from_json(&json);
        assert!(res.is_ok());
        let output_b = res.unwrap();

        assert_eq!(output_a, output_b)
    }
}
