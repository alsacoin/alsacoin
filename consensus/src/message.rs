//! # Message
//!
//! `message` is the module containing the consensus message type.

use crate::result::Result;
use crypto::hash::Digest;
use models::node::Node;
use models::transaction::Transaction;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;

/// `Message` is the type representing a consensus message type.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum Message {
    // NB: node is the sending node, not the receiving node
    FetchNodes {
        node: Node,
        count: u32,
        ids: Vec<Digest>,
    },
    FetchRandomNodes {
        node: Node,
        count: u32,
    },
    PushNodes {
        node: Node,
        count: u32,
        nodes: Vec<Node>,
    },
    FetchTransactions {
        node: Node,
        count: u32,
        ids: Vec<Digest>,
    },
    FetchRandomTransactions {
        node: Node,
        count: u32,
    },
    PushTransactions {
        node: Node,
        count: u32,
        transactions: Vec<Transaction>,
    },
    Query {
        node: Node,
        transaction: Transaction,
    },
    Reply {
        node: Node,
        id: Digest,
        chit: u8,
    },
}

impl Message {
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
