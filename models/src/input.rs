//! # Input
//!
//! `input` contains the `Input` type and functions.

use crate::address::Address;
use crate::error::Error;
use crate::result::Result;
use crypto::ecc::ed25519::{KeyPair, SecretKey, Signature};
use crypto::hash::{Blake512Hasher, Digest};
use crypto::random::Random;
use serde::{Deserialize, Serialize};

/// `Input` is an input in an Alsacoin `Transaction`.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct Input {
    pub address: Address,
    pub value: u64,
    pub signature: Option<Signature>,
    pub checksum: Digest,
}

impl Input {
    /// `new` creates a new unsigned `Input`.
    pub fn new(address: Address, value: u64) -> Result<Input> {
        let mut input = Input {
            address,
            value,
            signature: None,
            checksum: Digest::default(),
        };

        input.update_checksum()?;

        Ok(input)
    }

    /// `random` creates a random unsigned `Input`.
    pub fn random() -> Result<Input> {
        let address = Address::random()?;
        let value = Random::u64()?;

        Input::new(address, value)
    }

    /// `sign` calculates the input signature with a binary message.
    pub fn calc_sign(&self, secret_key: &SecretKey, msg: &[u8]) -> Result<Signature> {
        let kp = KeyPair::from_secret(secret_key)?;

        let mut clone = self.clone();
        clone.signature = None;
        clone.checksum = Digest::default();

        let mut buf = Vec::new();
        buf.extend_from_slice(msg);
        buf.extend_from_slice(&clone.to_bytes()?);

        kp.sign(&buf).map_err(|e| e.into())
    }

    /// `calc_checksum` calculates the `Input` checksum.
    pub fn calc_checksum(&self) -> Result<Digest> {
        let mut clone = self.clone();
        clone.checksum = Digest::default();

        let buf = clone.to_bytes()?;
        let checksum = Blake512Hasher::hash(&buf);
        Ok(checksum)
    }

    /// `sign` signs the `Input` and update its id.
    pub fn sign(&mut self, secret_key: &SecretKey, msg: &[u8]) -> Result<()> {
        self.signature = Some(self.calc_sign(secret_key, msg)?);

        Ok(())
    }

    /// `update_checksum` updates the `Input` checksum.
    pub fn update_checksum(&mut self) -> Result<()> {
        self.checksum = self.calc_checksum()?;

        Ok(())
    }

    /// `validate` validates the `Input`.
    pub fn validate(&self) -> Result<()> {
        if self.checksum != self.calc_checksum()? {
            let err = Error::InvalidChecksum;
            return Err(err);
        }

        Ok(())
    }

    /// `validate_signature` validates the `Input` signature.
    pub fn validate_signature(&self, secret_key: &SecretKey, msg: &[u8]) -> Result<()> {
        self.validate()?;

        if let Some(signature) = self.signature {
            if signature != self.calc_sign(secret_key, msg)? {
                let err = Error::InvalidSignature;
                return Err(err);
            }
        }

        Ok(())
    }

    /// `to_bytes` converts the `Input` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `Input`.
    pub fn from_bytes(b: &[u8]) -> Result<Input> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `Input` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `Input`.
    pub fn from_json(s: &str) -> Result<Input> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

#[test]
fn test_input_serialize_bytes() {
    for _ in 0..10 {
        let input_a = Input::random().unwrap();

        let res = input_a.to_bytes();
        assert!(res.is_ok());
        let cbor = res.unwrap();

        let res = Input::from_bytes(&cbor);
        assert!(res.is_ok());
        let input_b = res.unwrap();

        assert_eq!(input_a, input_b)
    }
}

#[test]
fn test_input_serialize_json() {
    for _ in 0..10 {
        let input_a = Input::random().unwrap();

        let res = input_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = Input::from_json(&json);
        assert!(res.is_ok());
        let input_b = res.unwrap();

        assert_eq!(input_a, input_b)
    }
}
