//! # Node
//!
//! `node` contains the Node model.

use crate::result::Result;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use crypto::random::Random;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;

/// Type representing a node in the distributed ledger network.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash, Serialize, Deserialize)]
pub struct Node {
    pub address: Vec<u8>,
    pub stage: Stage,
    pub last_seen: Timestamp,
}

impl Node {
    /// Creates a new `Node`.
    pub fn new(address: &[u8], stage: Stage) -> Node {
        Node {
            address: address.into(),
            stage,
            last_seen: Timestamp::now(),
        }
    }

    /// `random` creates a new random `Node`.
    pub fn random(address_len: usize) -> Result<Node> {
        let node = Node {
            address: Random::bytes(address_len)?,
            stage: Stage::random()?,
            last_seen: Timestamp::random()?,
        };

        Ok(node)
    }

    /// `validate` validates the `Node`.
    pub fn validate(&self) -> Result<()> {
        self.last_seen.validate()
    }

    /// `to_bytes` converts the `Node` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `Node`.
    pub fn from_bytes(b: &[u8]) -> Result<Node> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `Node` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `Node`.
    pub fn from_json(s: &str) -> Result<Node> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

#[test]
fn test_node_random() {
    let address_len = 100;

    for _ in 0..10 {
        let res = Node::random(address_len);
        assert!(res.is_ok());

        let node = res.unwrap();
        let res = node.validate();
        assert!(res.is_ok());
    }
}

#[test]
fn test_node_validate() {
    let address_len = 10;
    let address = Random::bytes(address_len).unwrap();;
    let stage = Stage::random().unwrap();
    let invalid_timestamp = Timestamp::new(2012, 12, 31, 12, 12, 12).unwrap();

    let mut node = Node::new(&address, stage);
    let res = node.validate();
    assert!(res.is_ok());

    node.last_seen = invalid_timestamp;
    let res = node.validate();
    assert!(res.is_err());
}

#[test]
fn test_node_serialize_bytes() {
    let address_len = 100;

    for _ in 0..10 {
        let node_a = Node::random(address_len).unwrap();

        let res = node_a.to_bytes();
        assert!(res.is_ok());
        let cbor = res.unwrap();

        let res = Node::from_bytes(&cbor);
        assert!(res.is_ok());
        let node_b = res.unwrap();

        assert_eq!(node_a, node_b)
    }
}

#[test]
fn test_node_serialize_json() {
    let address_len = 100;

    for _ in 0..10 {
        let node_a = Node::random(address_len).unwrap();

        let res = node_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = Node::from_json(&json);
        assert!(res.is_ok());
        let node_b = res.unwrap();

        assert_eq!(node_a, node_b)
    }
}
