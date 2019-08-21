//! # Message
//!
//! `message` contains the network message used in the crate.

use crate::result::Result;
use crypto::random::Random;
use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
/// `Message` is the network message used in the crate.
pub struct Message {
    pub address: Vec<u8>,
    pub data: Vec<u8>,
}

impl Message {
    /// `random` creates a random `Message`.
    pub fn random(address_len: u32, data_len: u32) -> Result<Message> {
        let msg = Message {
            address: Random::bytes(address_len as usize)?,
            data: Random::bytes(data_len as usize)?,
        };

        Ok(msg)
    }

    /// `to_bytes` converts the `Message` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `Message`.
    pub fn from_bytes(b: &[u8]) -> Result<Message> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `Message` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `Message`.
    pub fn from_json(s: &str) -> Result<Message> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

#[test]
fn test_message_serialize_bytes() {
    let address_len = 100;
    let data_len = 1000;

    for _ in 0..10 {
        let message_a = Message::random(address_len, data_len).unwrap();

        let res = message_a.to_bytes();
        assert!(res.is_ok());
        let cbor = res.unwrap();

        let res = Message::from_bytes(&cbor);
        assert!(res.is_ok());
        let message_b = res.unwrap();

        assert_eq!(message_a, message_b)
    }
}

#[test]
fn test_message_serialize_json() {
    let address_len = 100;
    let data_len = 1000;

    for _ in 0..10 {
        let message_a = Message::random(address_len, data_len).unwrap();

        let res = message_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = Message::from_json(&json);
        assert!(res.is_ok());
        let message_b = res.unwrap();

        assert_eq!(message_a, message_b)
    }
}
