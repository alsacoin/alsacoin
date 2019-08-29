//! # Consensus Config
//!
//! `consensus_config` is the module containing the consensus configuration type and functions.

use crate::result::Result;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;

/// `ConsensusConfig` is the type representing a consensus configuration.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub k: Option<u64>,
    pub alpha: Option<u64>,
    pub beta1: Option<u64>,
    pub beta2: Option<u64>,
}

impl ConsensusConfig {
    /// `DEFAULT_K` is the default consensus parameter k.
    /// See the Avalanche Consensus paper.
    pub const DEFAULT_K: u64 = 1; // TODO

    /// `DEFAULT_ALPHA` is the default consensus parameter alpha.
    /// See the Avalanche Consensus paper.
    pub const DEFAULT_ALPHA: u64 = 1; // TODO

    /// `DEFAULT_BETA1` is the default consensus parameter beta1.
    /// See the Avalanche Consensus paper.
    pub const DEFAULT_BETA1: u64 = 1; // TODO

    /// `DEFAULT_BETA2` is the default consensus parameter beta2.
    /// See the Avalanche Consensus paper.
    pub const DEFAULT_BETA2: u64 = 1; // TODO

    /// `new` creates a new `ConsensusConfig`.
    pub fn new(
        k: Option<u64>,
        alpha: Option<u64>,
        beta1: Option<u64>,
        beta2: Option<u64>,
    ) -> ConsensusConfig {
        let k = Some(k.unwrap_or(Self::DEFAULT_K));

        let alpha = Some(alpha.unwrap_or(Self::DEFAULT_ALPHA));

        let beta1 = Some(beta1.unwrap_or(Self::DEFAULT_BETA1));

        let beta2 = Some(beta2.unwrap_or(Self::DEFAULT_BETA2));

        ConsensusConfig {
            k,
            alpha,
            beta1,
            beta2,
        }
    }

    /// `to_bytes` converts the `ConsensusConfig` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `ConsensusConfig`.
    pub fn from_bytes(b: &[u8]) -> Result<ConsensusConfig> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `ConsensusConfig` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `ConsensusConfig`.
    pub fn from_json(s: &str) -> Result<ConsensusConfig> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

impl Default for ConsensusConfig {
    fn default() -> ConsensusConfig {
        let k = Some(ConsensusConfig::DEFAULT_K);
        let alpha = Some(ConsensusConfig::DEFAULT_ALPHA);
        let beta1 = Some(ConsensusConfig::DEFAULT_BETA1);
        let beta2 = Some(ConsensusConfig::DEFAULT_BETA2);

        ConsensusConfig {
            k,
            alpha,
            beta1,
            beta2,
        }
    }
}

#[test]
fn test_consensus_config_serialize_bytes() {
    let config_a = ConsensusConfig::default();

    let res = config_a.to_bytes();
    assert!(res.is_ok());
    let cbor = res.unwrap();

    let res = ConsensusConfig::from_bytes(&cbor);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}

#[test]
fn test_consensus_config_serialize_json() {
    let config_a = ConsensusConfig::default();

    let res = config_a.to_json();
    assert!(res.is_ok());
    let json = res.unwrap();

    let res = ConsensusConfig::from_json(&json);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}
