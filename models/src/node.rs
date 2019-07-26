//! # Node
//!
//! `node` contains the Node model.

use crate::result::Result;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use serde::{Deserialize, Serialize};

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

    pub fn validate(&self) -> Result<()> {
        self.last_seen.validate()
    }
}

#[test]
fn test_node_validate() {
    let address = [1, 1, 1, 1];
    let stage = Stage::default();
    let invalid_timestamp = Timestamp::from_date(2012, 12, 31, 12, 12, 12).unwrap();

    let mut node = Node::new(&address, stage);
    let res = node.validate();
    assert!(res.is_ok());

    node.last_seen = invalid_timestamp;
    let res = node.validate();
    assert!(res.is_err());
}
