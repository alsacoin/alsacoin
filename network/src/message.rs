//! # Message
//!
//! `message` contains the network message used in the crate.

use crate::error::Error;
use crate::result::Result;
use crypto::random::Random;
use models::consensus_message::ConsensusMessage;
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

    /// `from_consensus_message` creates a `Message` from a `ConsensusMessage`.
    pub fn from_consensus_message(cons_msg: &ConsensusMessage) -> Result<Message> {
        cons_msg.validate()?;

        let address = cons_msg.node().address;
        let data = cons_msg.to_bytes()?;

        let msg = Message { address, data };

        Ok(msg)
    }

    /// `to_consensus_message` converts the `Message` to a `ConsensusMessage`.
    pub fn to_consensus_message(&self) -> Result<ConsensusMessage> {
        let cons_msg = ConsensusMessage::from_bytes(&self.data)?;
        cons_msg.validate()?;

        if cons_msg.node().address != self.address {
            let err = Error::InvalidAddress;
            return Err(err);
        }

        Ok(cons_msg)
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
fn test_message_consensus_message() {
    use crypto::hash::Digest;
    use models::node::Node;

    let address_len = 100;
    let node = Node::random(address_len).unwrap();
    let query_id = Random::u64().unwrap();
    let tx_id = Digest::random().unwrap();
    let chit = Random::u32_range(0, 2).unwrap() != 0;

    let cons_msg_a = ConsensusMessage::new_reply(query_id, &node, tx_id, chit).unwrap();

    let res = Message::from_consensus_message(&cons_msg_a);
    assert!(res.is_ok());

    let msg = res.unwrap();

    let res = msg.to_consensus_message();
    assert!(res.is_ok());

    let cons_msg_b = res.unwrap();

    assert_eq!(cons_msg_a, cons_msg_b)
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
