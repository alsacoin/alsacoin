//! # Transaction
//!
//! `transaction` contains the `Transaction` type and functions.

use crate::address::Address;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use crate::version::Version;
use crypto::ecc::ed25519::Signature;
use crypto::hash::Digest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// `Transaction` is the Alsacoin transaction type. It is built
/// around the HybridTx model defined in `Chimeric Ledgers` papers.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Digest,
    pub version: Version,
    pub stage: Stage,
    pub time: Timestamp,
    pub locktime: Timestamp,
    pub inputs: BTreeMap<Address, (u64, Signature)>,
    pub outputs: BTreeMap<Address, u64>,
    pub fee: u64,
    pub nonce: u64,
}
