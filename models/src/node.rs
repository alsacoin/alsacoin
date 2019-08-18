//! # Node
//!
//! `node` contains the Node model.

use crate::error::Error;
use crate::result::Result;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use crate::traits::Storable;
use crypto::hash::{Blake512Hasher, Digest};
use crypto::random::Random;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use store::traits::Store;

/// Type representing a node in the distributed ledger network.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
pub struct Node {
    pub id: Digest,
    pub address: Vec<u8>,
    pub stage: Stage,
    pub last_seen: Timestamp,
}

impl Node {
    /// Creates a new `Node`.
    pub fn new(address: &[u8], stage: Stage) -> Node {
        let hash = Blake512Hasher::hash(address);

        Node {
            id: hash,
            address: address.into(),
            stage,
            last_seen: Timestamp::now(),
        }
    }

    /// `random` creates a new random `Node`.
    pub fn random(address_len: usize) -> Result<Node> {
        let address = Random::bytes(address_len)?;
        let id = Blake512Hasher::hash(&address);

        let node = Node {
            id,
            address,
            stage: Stage::random()?,
            last_seen: Timestamp::now(),
        };

        Ok(node)
    }

    /// `calc_id` calculates the `Node` id.
    pub fn calc_id(&self) -> Digest {
        Blake512Hasher::hash(&self.address)
    }

    /// `validate` validates the `Node`.
    pub fn validate(&self) -> Result<()> {
        if self.id != self.calc_id() {
            let err = Error::InvalidId;
            return Err(err);
        }

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
    const KEY_PREFIX: u8 = 1;

    type Key = Digest;

    fn key_to_bytes(key: &Self::Key) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(<Self as Storable<S>>::KEY_PREFIX);
        buf.extend_from_slice(&key.to_bytes());
        Ok(buf)
    }

    fn lookup(store: &S, key: &Self::Key) -> Result<bool> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        store.lookup(&key).map_err(|e| e.into())
    }

    fn get(store: &S, key: &Self::Key) -> Result<Self> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        let buf = store.get(&key)?;
        Self::from_bytes(&buf)
    }

    fn query(
        store: &S,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<Vec<Self>> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(key)?;
            Some(key)
        } else {
            None
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(key)?;
            Some(key)
        } else {
            None
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        let values = store.query(from, to, count, skip)?;
        let mut items = Vec::new();

        for value in values {
            let item = Self::from_bytes(&value)?;
            items.push(item);
        }

        Ok(items)
    }

    fn count(
        store: &S,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        skip: Option<u32>,
    ) -> Result<u32> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(key)?;
            Some(key)
        } else {
            None
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(key)?;
            Some(key)
        } else {
            None
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        store.count(from, to, skip).map_err(|e| e.into())
    }

    fn insert(store: &mut S, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        let value = value.to_bytes()?;
        store.insert(&key, &value).map_err(|e| e.into())
    }

    fn create(store: &mut S, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        let value = value.to_bytes()?;
        store.create(&key, &value).map_err(|e| e.into())
    }

    fn update(store: &mut S, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        let value = value.to_bytes()?;
        store.update(&key, &value).map_err(|e| e.into())
    }

    fn insert_batch(store: &mut S, items: &[(Self::Key, Self)]) -> Result<()> {
        let mut _items = Vec::new();

        for (k, v) in items {
            let k = <Self as Storable<S>>::key_to_bytes(k)?;
            let v = v.to_bytes()?;
            let item = (k, v);
            _items.push(item);
        }

        let items: Vec<(&[u8], &[u8])> = _items
            .iter()
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
            .collect();

        store.insert_batch(&items).map_err(|e| e.into())
    }

    fn remove(store: &mut S, key: &Self::Key) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        store.remove(&key).map_err(|e| e.into())
    }

    fn remove_batch(store: &mut S, keys: &[Self::Key]) -> Result<()> {
        let mut _keys = Vec::new();
        for key in keys {
            let key = <Self as Storable<S>>::key_to_bytes(key)?;
            _keys.push(key);
        }

        let keys: Vec<&[u8]> = _keys.iter().map(|k| k.as_slice()).collect();

        store.remove_batch(&keys).map_err(|e| e.into())
    }

    fn cleanup(store: &mut S, min_time: Timestamp) -> Result<()> {
        let mut _from = Digest::default();
        _from[0] = <Self as Storable<S>>::KEY_PREFIX;
        let from = Some(_from);

        let mut _to = Digest::default();
        _to[0] = <Self as Storable<S>>::KEY_PREFIX + 1;
        let to = Some(_to);

        for item in <Self as Storable<S>>::query(store, from, to, None, None)? {
            if item.last_seen < min_time {
                <Self as Storable<S>>::remove(store, &item.id)?;
            }
        }

        Ok(())
    }

    fn clear(store: &mut S) -> Result<()> {
        let from = Some(vec![<Self as Storable<S>>::KEY_PREFIX]);
        let from = from.as_ref().map(|from| from.as_slice());
        let to = Some(vec![<Self as Storable<S>>::KEY_PREFIX + 1]);
        let to = to.as_ref().map(|to| to.as_slice());
        store.remove_range(from, to, None).map_err(|e| e.into())
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

    node.last_seen = Timestamp::default();
    node.id = Digest::default();
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
