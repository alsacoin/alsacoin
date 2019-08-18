//! # Node
//!
//! `node` contains the Node model.

use crate::result::Result;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use crate::traits::Storable;
use crypto::random::Random;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use store::traits::Store;

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
            last_seen: Timestamp::now(),
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

impl<S: Store> Storable<S> for Node {
    const KEY_PREFIX: u32 = 1;

    type Key = Vec<u8>;

    fn lookup(&self, _store: &S, _key: &Self::Key) -> Result<bool> {
        // TODO
        unreachable!()
    }

    fn get(&self, _store: &S, _key: &Self::Key) -> Result<Self> {
        // TODO
        unreachable!()
    }

    fn query(
        &self,
        _store: &S,
        _from: Option<&Self::Key>,
        _to: Option<&Self::Key>,
        _count: Option<u32>,
        _skip: Option<u32>,
    ) -> Result<Vec<Self>> {
        // TODO
        unreachable!()
    }

    fn count(
        &self,
        _store: &S,
        _from: Option<&Self::Key>,
        _to: Option<&Self::Key>,
        _skip: Option<u32>,
    ) -> Result<u32> {
        // TODO
        unreachable!()
    }

    fn insert(&mut self, _store: &mut S, _key: &Self::Key, _value: &Self) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn create(&mut self, _store: &mut S, _key: &Self::Key, _value: &Self) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn update(&mut self, _store: &mut S, _key: &Self::Key, _value: &Self) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn insert_batch(&mut self, _store: &mut S, _items: &[(Self::Key, Self)]) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn remove(&mut self, _store: &mut S, _key: &Self::Key) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn remove_batch(&mut self, _store: &mut S, _keys: &[Self::Key]) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn cleanup(&mut self, _store: &mut S) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn clear(&mut self, _store: &mut S) -> Result<()> {
        // TODO
        unreachable!()
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
