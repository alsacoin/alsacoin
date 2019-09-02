//! # Consensus Message
//!
//! `consensus_message` is the module containing the consensus message type.

use crate::error::Error;
use crate::node::Node;
use crate::result::Result;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use crate::traits::Storable;
use crate::transaction::Transaction;
use byteorder::{BigEndian, WriteBytesExt};
use crypto::hash::Digest;
use crypto::random::Random;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use std::collections::BTreeSet;
use store::traits::Store;

/// `ConsensusMessage` is the type representing a consensus message type.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum ConsensusMessage {
    // NB: node is the sending node, not the receiving node
    Ping {
        id: u64,
        node: Node,
    },
    FetchNodes {
        id: u64,
        node: Node,
        count: u32,
        ids: BTreeSet<Digest>,
    },
    FetchRandomNodes {
        id: u64,
        node: Node,
        count: u32,
    },
    PushNodes {
        id: u64,
        node: Node,
        count: u32,
        ids: BTreeSet<Digest>,
        nodes: BTreeSet<Node>,
    },
    FetchTransactions {
        id: u64,
        node: Node,
        count: u32,
        ids: BTreeSet<Digest>,
    },
    FetchRandomTransactions {
        id: u64,
        node: Node,
        count: u32,
    },
    PushTransactions {
        id: u64,
        node: Node,
        count: u32,
        ids: BTreeSet<Digest>,
        transactions: BTreeSet<Transaction>,
    },
    Query {
        id: u64,
        node: Node,
        transaction: Transaction,
    },
    Reply {
        id: u64,
        node: Node,
        tx_id: Digest,
        chit: bool,
    },
}

impl ConsensusMessage {
    /// `new_ping` creates a new `Ping` `ConsensusMessage`.
    pub fn new_ping(node: &Node) -> Result<ConsensusMessage> {
        node.validate()?;

        let message = ConsensusMessage::Ping {
            id: Random::u64()?,
            node: node.to_owned(),
        };

        Ok(message)
    }

    /// `new_fetch_nodes` creates a new `FetchNodes` `ConsensusMessage`.
    pub fn new_fetch_nodes(node: &Node, ids: &BTreeSet<Digest>) -> Result<ConsensusMessage> {
        node.validate()?;

        if ids.contains(&node.id) {
            let err = Error::InvalidId;
            return Err(err);
        }

        let message = ConsensusMessage::FetchNodes {
            id: Random::u64()?,
            node: node.to_owned(),
            count: ids.len() as u32,
            ids: ids.to_owned(),
        };

        Ok(message)
    }

    /// `new_fetch_random_nodes` creates a new `FetchRandomNodes` `ConsensusMessage`.
    pub fn new_fetch_random_nodes(node: &Node, count: u32) -> Result<ConsensusMessage> {
        node.validate()?;

        let message = ConsensusMessage::FetchRandomNodes {
            id: Random::u64()?,
            node: node.to_owned(),
            count,
        };

        Ok(message)
    }

    /// `new_push_nodes` creates a new `PushNodes` `ConsensusMessage`.
    pub fn new_push_nodes(
        fetch_id: u64,
        node: &Node,
        nodes: &BTreeSet<Node>,
    ) -> Result<ConsensusMessage> {
        node.validate()?;

        for node in nodes.iter() {
            node.validate()?;
        }

        let ids: BTreeSet<Digest> = nodes.iter().map(|tx| tx.id).collect();

        let count = ids.len() as u32;

        let message = ConsensusMessage::PushNodes {
            id: fetch_id + 1,
            node: node.to_owned(),
            count,
            ids: ids.to_owned(),
            nodes: nodes.to_owned(),
        };

        Ok(message)
    }

    /// `new_fetch_transactions` creates a new `FetchTransactions` `ConsensusMessage`.
    pub fn new_fetch_transactions(node: &Node, ids: &BTreeSet<Digest>) -> Result<ConsensusMessage> {
        node.validate()?;

        if ids.contains(&node.id) {
            let err = Error::InvalidId;
            return Err(err);
        }

        let message = ConsensusMessage::FetchTransactions {
            id: Random::u64()?,
            node: node.to_owned(),
            count: ids.len() as u32,
            ids: ids.to_owned(),
        };

        Ok(message)
    }

    /// `new_fetch_random_transactions` creates a new `FetchRandomTransactions` `ConsensusMessage`.
    pub fn new_fetch_random_transactions(node: &Node, count: u32) -> Result<ConsensusMessage> {
        node.validate()?;

        let message = ConsensusMessage::FetchRandomTransactions {
            id: Random::u64()?,
            node: node.to_owned(),
            count,
        };

        Ok(message)
    }

    /// `new_push_transactions` creates a new `PushTransactions` `ConsensusMessage`.
    pub fn new_push_transactions(
        fetch_id: u64,
        node: &Node,
        transactions: &BTreeSet<Transaction>,
    ) -> Result<ConsensusMessage> {
        node.validate()?;

        for transaction in transactions.iter() {
            transaction.validate()?;
        }

        let ids: BTreeSet<Digest> = transactions.iter().map(|tx| tx.id).collect();

        let count = ids.len() as u32;

        let message = ConsensusMessage::PushTransactions {
            id: fetch_id + 1,
            node: node.to_owned(),
            count,
            ids: ids.to_owned(),
            transactions: transactions.to_owned(),
        };

        Ok(message)
    }

    /// `new_query` creates a new `Query` `ConsensusMessage`.
    pub fn new_query(node: &Node, transaction: &Transaction) -> Result<ConsensusMessage> {
        node.validate()?;
        transaction.validate()?;

        let message = ConsensusMessage::Query {
            id: Random::u64()?,
            node: node.to_owned(),
            transaction: transaction.to_owned(),
        };

        Ok(message)
    }

    /// `new_reply` creates a new `Reply` `ConsensusMessage`.
    pub fn new_reply(
        query_id: u64,
        node: &Node,
        tx_id: Digest,
        chit: bool,
    ) -> Result<ConsensusMessage> {
        node.validate()?;

        if tx_id == node.id {
            let err = Error::InvalidId;
            return Err(err);
        }

        let message = ConsensusMessage::Reply {
            id: query_id + 1,
            node: node.to_owned(),
            tx_id,
            chit,
        };

        Ok(message)
    }

    /// `id` returns the `ConsensusMessage` id.
    pub fn id(&self) -> u64 {
        match self {
            ConsensusMessage::Ping { id, .. } => *id,
            ConsensusMessage::FetchNodes { id, .. } => *id,
            ConsensusMessage::FetchRandomNodes { id, .. } => *id,
            ConsensusMessage::PushNodes { id, .. } => *id,
            ConsensusMessage::FetchTransactions { id, .. } => *id,
            ConsensusMessage::FetchRandomTransactions { id, .. } => *id,
            ConsensusMessage::PushTransactions { id, .. } => *id,
            ConsensusMessage::Query { id, .. } => *id,
            ConsensusMessage::Reply { id, .. } => *id,
        }
    }

    /// `node` returns the `ConsensusMessage` `Node`.
    pub fn node(&self) -> Node {
        match self {
            ConsensusMessage::Ping { node, .. } => node.clone(),
            ConsensusMessage::FetchNodes { node, .. } => node.clone(),
            ConsensusMessage::FetchRandomNodes { node, .. } => node.clone(),
            ConsensusMessage::PushNodes { node, .. } => node.clone(),
            ConsensusMessage::FetchTransactions { node, .. } => node.clone(),
            ConsensusMessage::FetchRandomTransactions { node, .. } => node.clone(),
            ConsensusMessage::PushTransactions { node, .. } => node.clone(),
            ConsensusMessage::Query { node, .. } => node.clone(),
            ConsensusMessage::Reply { node, .. } => node.clone(),
        }
    }

    /// `validate_ping` validates a `FetchTransactions`
    /// `ConsensusMessage`.
    pub fn validate_ping(&self) -> Result<()> {
        match self {
            ConsensusMessage::Ping { node, .. } => {
                node.validate()?;

                Ok(())
            }
            _ => Err(Error::InvalidMessage),
        }
    }

    /// `validate_fetch_nodes` validates a `FetchTransactions`
    /// `ConsensusMessage`.
    pub fn validate_fetch_nodes(&self) -> Result<()> {
        match self {
            ConsensusMessage::FetchNodes {
                node, count, ids, ..
            } => {
                node.validate()?;

                if ids.len() as u32 != *count {
                    let err = Error::InvalidLength;
                    return Err(err);
                }

                if ids.contains(&node.id) {
                    let err = Error::InvalidId;
                    return Err(err);
                }

                Ok(())
            }
            _ => Err(Error::InvalidMessage),
        }
    }

    /// `validate_fetch_random_nodes` validates a `FetchRandomTransactions`
    /// `ConsensusMessage`.
    pub fn validate_fetch_random_nodes(&self) -> Result<()> {
        match self {
            ConsensusMessage::FetchRandomNodes { node, .. } => node.validate(),
            _ => Err(Error::InvalidMessage),
        }
    }

    /// `validate_push_nodes` validates a `PushTransactions`
    /// `ConsensusMessage`.
    pub fn validate_push_nodes(&self) -> Result<()> {
        match self {
            ConsensusMessage::PushNodes {
                node,
                count,
                ids,
                nodes,
                ..
            } => {
                node.validate()?;

                for node in nodes.iter() {
                    node.validate()?;
                }

                if ids.contains(&node.id) {
                    let err = Error::InvalidId;
                    return Err(err);
                }

                if ids.len() as u32 != *count {
                    let err = Error::InvalidLength;
                    return Err(err);
                }

                if ids.len() != nodes.len() {
                    let err = Error::InvalidLength;
                    return Err(err);
                }

                let found_ids: BTreeSet<Digest> = nodes.iter().map(|node| node.id).collect();

                if ids != &found_ids {
                    let err = Error::InvalidTransactions;
                    return Err(err);
                }

                Ok(())
            }
            _ => Err(Error::InvalidMessage),
        }
    }

    /// `validate_fetch_transactions` validates a `FetchTransactions`
    /// `ConsensusMessage`.
    pub fn validate_fetch_transactions(&self) -> Result<()> {
        match self {
            ConsensusMessage::FetchTransactions {
                node, count, ids, ..
            } => {
                node.validate()?;

                if ids.len() as u32 != *count {
                    let err = Error::InvalidLength;
                    return Err(err);
                }

                if ids.contains(&node.id) {
                    let err = Error::InvalidId;
                    return Err(err);
                }

                Ok(())
            }
            _ => Err(Error::InvalidMessage),
        }
    }

    /// `validate_fetch_random_transactions` validates a `FetchRandomTransactions`
    /// `ConsensusMessage`.
    pub fn validate_fetch_random_transactions(&self) -> Result<()> {
        match self {
            ConsensusMessage::FetchRandomTransactions { node, .. } => node.validate(),
            _ => Err(Error::InvalidMessage),
        }
    }

    /// `validate_push_transactions` validates a `PushTransactions`
    /// `ConsensusMessage`.
    pub fn validate_push_transactions(&self) -> Result<()> {
        match self {
            ConsensusMessage::PushTransactions {
                node,
                count,
                ids,
                transactions,
                ..
            } => {
                node.validate()?;

                for transaction in transactions.iter() {
                    transaction.validate()?;
                }

                if ids.contains(&node.id) {
                    let err = Error::InvalidId;
                    return Err(err);
                }

                if ids.len() as u32 != *count {
                    let err = Error::InvalidLength;
                    return Err(err);
                }

                if transactions.len() as u32 != *count {
                    let err = Error::InvalidLength;
                    return Err(err);
                }

                let found_ids: BTreeSet<Digest> = transactions.iter().map(|tx| tx.id).collect();

                if ids != &found_ids {
                    let err = Error::InvalidTransactions;
                    return Err(err);
                }

                Ok(())
            }
            _ => Err(Error::InvalidMessage),
        }
    }

    /// `validate_query` validates a `Query` `ConsensusMessage`.
    pub fn validate_query(&self) -> Result<()> {
        match self {
            ConsensusMessage::Query {
                node, transaction, ..
            } => {
                node.validate()?;
                transaction.validate()
            }
            _ => Err(Error::InvalidMessage),
        }
    }

    /// `validate_reply` validates a `Reply` `ConsensusMessage`.
    pub fn validate_reply(&self) -> Result<()> {
        match self {
            ConsensusMessage::Reply { node, tx_id, .. } => {
                node.validate()?;

                if tx_id == &node.id {
                    let err = Error::InvalidId;
                    return Err(err);
                }

                Ok(())
            }
            _ => Err(Error::InvalidMessage),
        }
    }

    /// `is_ping` returns if the `ConsensusMessage` is a `Ping` message.
    pub fn is_ping(&self) -> Result<bool> {
        self.validate()?;

        let res = match self {
            ConsensusMessage::Ping { .. } => true,
            _ => false,
        };

        Ok(res)
    }

    /// `is_fetch_nodes` returns if the `ConsensusMessage` is a `FetchNodes` message.
    pub fn is_fetch_nodes(&self) -> Result<bool> {
        self.validate()?;

        let res = match self {
            ConsensusMessage::FetchNodes { .. } => true,
            _ => false,
        };

        Ok(res)
    }

    /// `is_fetch_random_nodes` returns if the `ConsensusMessage` is a
    /// `FetchRandomNodes` message.
    pub fn is_fetch_random_nodes(&self) -> Result<bool> {
        self.validate()?;

        let res = match self {
            ConsensusMessage::FetchRandomNodes { .. } => true,
            _ => false,
        };

        Ok(res)
    }

    /// `is_push_nodes` returns if the `ConsensusMessage` is a `PushNodes` message.
    pub fn is_push_nodes(&self) -> Result<bool> {
        self.validate()?;

        let res = match self {
            ConsensusMessage::PushNodes { .. } => true,
            _ => false,
        };

        Ok(res)
    }

    /// `is_fetch_transactions` returns if the `ConsensusMessage` is a `FetchTransactions` message.
    pub fn is_fetch_transactions(&self) -> Result<bool> {
        self.validate()?;

        let res = match self {
            ConsensusMessage::FetchTransactions { .. } => true,
            _ => false,
        };

        Ok(res)
    }

    /// `is_fetch_random_transactions` returns if the `ConsensusMessage` is a
    /// `FetchRandomTransactions` message.
    pub fn is_fetch_random_transactions(&self) -> Result<bool> {
        self.validate()?;

        let res = match self {
            ConsensusMessage::FetchRandomTransactions { .. } => true,
            _ => false,
        };

        Ok(res)
    }

    /// `is_push_transactions` returns if the `ConsensusMessage` is a `PushTransactions` message.
    pub fn is_push_transactions(&self) -> Result<bool> {
        self.validate()?;

        let res = match self {
            ConsensusMessage::PushTransactions { .. } => true,
            _ => false,
        };

        Ok(res)
    }

    /// `is_query` returns if the `ConsensusMessage` is a `Query` message.
    pub fn is_query(&self) -> Result<bool> {
        self.validate()?;

        let res = match self {
            ConsensusMessage::Query { .. } => true,
            _ => false,
        };

        Ok(res)
    }

    /// `is_reply` returns if the `ConsensusMessage` is a `Reply` message.
    pub fn is_reply(&self) -> Result<bool> {
        self.validate()?;

        let res = match self {
            ConsensusMessage::Reply { .. } => true,
            _ => false,
        };

        Ok(res)
    }

    /// `validate` validates a `ConsensusMessage`.
    pub fn validate(&self) -> Result<()> {
        match self {
            ConsensusMessage::Ping { .. } => self.validate_ping(),
            ConsensusMessage::FetchNodes { .. } => self.validate_fetch_nodes(),
            ConsensusMessage::FetchRandomNodes { .. } => self.validate_fetch_random_nodes(),
            ConsensusMessage::PushNodes { .. } => self.validate_push_nodes(),
            ConsensusMessage::FetchTransactions { .. } => self.validate_fetch_transactions(),
            ConsensusMessage::FetchRandomTransactions { .. } => {
                self.validate_fetch_random_transactions()
            }
            ConsensusMessage::PushTransactions { .. } => self.validate_push_transactions(),
            ConsensusMessage::Query { .. } => self.validate_query(),
            ConsensusMessage::Reply { .. } => self.validate_reply(),
        }
    }

    /// `to_bytes` converts the `ConsensusMessage` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `ConsensusMessage`.
    pub fn from_bytes(b: &[u8]) -> Result<ConsensusMessage> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `ConsensusMessage` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `ConsensusMessage`.
    pub fn from_json(s: &str) -> Result<ConsensusMessage> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

impl<S: Store> Storable<S> for ConsensusMessage {
    const KEY_PREFIX: u8 = 7;

    type Key = u64;

    fn key_to_bytes(stage: Stage, key: &Self::Key) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(stage as u8);
        buf.push(<Self as Storable<S>>::KEY_PREFIX);
        buf.write_u64::<BigEndian>(*key)?;
        Ok(buf)
    }

    fn lookup(store: &S, stage: Stage, key: &Self::Key) -> Result<bool> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        store.lookup(&key).map_err(|e| e.into())
    }

    fn get(store: &S, stage: Stage, key: &Self::Key) -> Result<Self> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        let buf = store.get(&key)?;
        Self::from_bytes(&buf)
    }

    fn query(
        store: &S,
        stage: Stage,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<BTreeSet<Self>> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            None
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            None
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        let values = store.query(from, to, count, skip)?;
        let mut items = BTreeSet::new();

        for value in values {
            let item = Self::from_bytes(&value)?;
            items.insert(item);
        }

        Ok(items)
    }

    fn sample(
        store: &S,
        stage: Stage,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        count: u32,
    ) -> Result<BTreeSet<Self>> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            None
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            None
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        let values = store.sample(from, to, count)?;
        let mut items = BTreeSet::new();

        for value in values {
            let item = Self::from_bytes(&value)?;
            items.insert(item);
        }

        Ok(items)
    }

    fn count(
        store: &S,
        stage: Stage,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        skip: Option<u32>,
    ) -> Result<u32> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            None
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            None
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        store.count(from, to, skip).map_err(|e| e.into())
    }

    fn insert(store: &mut S, stage: Stage, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        let value = value.to_bytes()?;
        store.insert(&key, &value).map_err(|e| e.into())
    }

    fn create(store: &mut S, stage: Stage, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        let value = value.to_bytes()?;
        store.create(&key, &value).map_err(|e| e.into())
    }

    fn update(store: &mut S, stage: Stage, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        let value = value.to_bytes()?;
        store.update(&key, &value).map_err(|e| e.into())
    }

    fn insert_batch(store: &mut S, stage: Stage, items: &[(Self::Key, Self)]) -> Result<()> {
        let mut _items = BTreeSet::new();

        for (k, v) in items {
            let k = <Self as Storable<S>>::key_to_bytes(stage, k)?;
            let v = v.to_bytes()?;
            let item = (k, v);
            _items.insert(item);
        }

        let items: Vec<(&[u8], &[u8])> = _items
            .iter()
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
            .collect();

        store.insert_batch(&items).map_err(|e| e.into())
    }

    fn remove(store: &mut S, stage: Stage, key: &Self::Key) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        store.remove(&key).map_err(|e| e.into())
    }

    fn remove_batch(store: &mut S, stage: Stage, keys: &[Self::Key]) -> Result<()> {
        let mut _keys = BTreeSet::new();
        for key in keys {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            _keys.insert(key);
        }

        let keys: Vec<&[u8]> = _keys.iter().map(|k| k.as_slice()).collect();

        store.remove_batch(&keys).map_err(|e| e.into())
    }

    fn cleanup(_store: &mut S, _stage: Stage, _min_time: Timestamp) -> Result<()> {
        Err(Error::NotImplemented)
    }

    fn clear(store: &mut S, stage: Stage) -> Result<()> {
        let from = Some(vec![stage as u8, <Self as Storable<S>>::KEY_PREFIX]);
        let from = from.as_ref().map(|from| from.as_slice());

        let to = Some(vec![stage as u8, <Self as Storable<S>>::KEY_PREFIX + 1]);
        let to = to.as_ref().map(|to| to.as_slice());

        store.remove_range(from, to, None).map_err(|e| e.into())
    }
}

#[test]
fn test_consensus_message() {
    let address_len = 100;
    let node = Node::random(address_len).unwrap();
    let query_id = Random::u64().unwrap();
    let tx_id = Digest::random().unwrap();
    let chit = Random::u32_range(0, 2).unwrap() != 0;

    let mut invalid_node = node.clone();
    invalid_node.id = Digest::default();

    let res = ConsensusMessage::new_reply(query_id, &invalid_node, tx_id, chit);
    assert!(res.is_err());

    let res = ConsensusMessage::new_reply(query_id, &node, node.id, chit);
    assert!(res.is_err());

    let res = ConsensusMessage::new_reply(query_id, &node, tx_id, chit);
    assert!(res.is_ok());

    let cons_msg = res.unwrap();

    let res = cons_msg.validate_query();
    assert!(res.is_err());

    let res = cons_msg.validate_reply();
    assert!(res.is_ok());

    let res = cons_msg.validate();
    assert!(res.is_ok());

    let cons_msg = ConsensusMessage::Reply {
        id: query_id,
        node: invalid_node.clone(),
        tx_id,
        chit,
    };

    let res = cons_msg.validate();
    assert!(res.is_err());

    let res = cons_msg.validate_reply();
    assert!(res.is_err());
}

#[test]
fn test_consensus_message_serialize_bytes() {
    let address_len = 100;

    for _ in 0..10 {
        let node = Node::random(address_len).unwrap();
        let query_id = Random::u64().unwrap();
        let tx_id = Digest::random().unwrap();
        let chit = Random::u32_range(0, 2).unwrap() != 0;

        let cons_msg_a = ConsensusMessage::new_reply(query_id, &node, tx_id, chit).unwrap();

        let res = cons_msg_a.to_bytes();
        assert!(res.is_ok());
        let cbor = res.unwrap();

        let res = ConsensusMessage::from_bytes(&cbor);
        assert!(res.is_ok());
        let cons_msg_b = res.unwrap();

        assert_eq!(cons_msg_a, cons_msg_b)
    }
}

#[test]
fn test_consensus_message_serialize_json() {
    let address_len = 100;

    for _ in 0..10 {
        let node = Node::random(address_len).unwrap();
        let query_id = Random::u64().unwrap();
        let tx_id = Digest::random().unwrap();
        let chit = Random::u32_range(0, 2).unwrap() != 0;

        let cons_msg_a = ConsensusMessage::new_reply(query_id, &node, tx_id, chit).unwrap();

        let res = cons_msg_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = ConsensusMessage::from_json(&json);
        assert!(res.is_ok());
        let cons_msg_b = res.unwrap();

        assert_eq!(cons_msg_a, cons_msg_b)
    }
}

#[test]
fn test_consensus_message_storable() {
    use store::memory::MemoryStoreFactory;

    let max_value_size = 1 << 10;
    let max_size = 1 << 30;

    let mut store = MemoryStoreFactory::new_unqlite(max_value_size, max_size).unwrap();

    let address_len = 100;
    let stage = Stage::random().unwrap();

    let items: Vec<(u64, ConsensusMessage)> = (0..10)
        .map(|query_id| {
            let address = Random::bytes(address_len).unwrap();
            let node = Node::new(stage, &address);
            let tx_id = Digest::random().unwrap();
            let chit = Random::u32_range(0, 2).unwrap() != 0;

            let cons_msg = ConsensusMessage::new_reply(query_id, &node, tx_id, chit).unwrap();
            (query_id, cons_msg)
        })
        .collect();

    for (key, value) in &items {
        let res = ConsensusMessage::count(&store, stage, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = ConsensusMessage::query(&store, stage, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0);

        let res = ConsensusMessage::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = ConsensusMessage::get(&store, stage, &key);
        assert!(res.is_err());

        let res = ConsensusMessage::insert(&mut store, stage, &key, &value);
        assert!(res.is_ok());

        let res = ConsensusMessage::count(&store, stage, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);

        let res = ConsensusMessage::query(&store, stage, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().iter().next(), Some(value));

        let res = ConsensusMessage::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(found);

        let res = ConsensusMessage::get(&store, stage, &key);
        assert!(res.is_ok());
        assert_eq!(&res.unwrap(), value);

        let res = ConsensusMessage::remove(&mut store, stage, &key);
        assert!(res.is_ok());

        let res = ConsensusMessage::count(&store, stage, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = ConsensusMessage::query(&store, stage, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0);

        let res = ConsensusMessage::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = ConsensusMessage::get(&store, stage, &key);
        assert!(res.is_err());

        let res = ConsensusMessage::insert(&mut store, stage, &key, &value);
        assert!(res.is_ok());

        let res = ConsensusMessage::clear(&mut store, stage);
        assert!(res.is_ok());

        let res = ConsensusMessage::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);
    }
}
