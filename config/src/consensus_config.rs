//! # Consensus Config
//!
//! `consensus_config` is the module containing the consensus configuration type and functions.

use crate::result::Result;
use crypto::hash::balloon::BalloonParams;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use toml;

/// `ConsensusConfig` is the type representing a consensus configuration.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub k: Option<u32>,
    pub alpha: Option<u32>,
    pub beta1: Option<u32>,
    pub beta2: Option<u32>,
    pub s_cost: Option<u32>,
    pub t_cost: Option<u32>,
    pub delta: Option<u32>,
    pub max_retries: Option<u32>,
    pub timeout: Option<u64>,
}

impl ConsensusConfig {
    /// `DEFAULT_K` is the default consensus parameter k.
    /// See the Avalanche Consensus paper.
    pub const DEFAULT_K: u32 = 1; // TODO

    /// `DEFAULT_ALPHA` is the default consensus parameter alpha.
    /// See the Avalanche Consensus paper.
    pub const DEFAULT_ALPHA: u32 = 1; // TODO

    /// `DEFAULT_BETA1` is the default consensus parameter beta1.
    /// See the Avalanche Consensus paper.
    pub const DEFAULT_BETA1: u32 = 1; // TODO

    /// `DEFAULT_BETA2` is the default consensus parameter beta2.
    /// See the Avalanche Consensus paper.
    pub const DEFAULT_BETA2: u32 = 1; // TODO

    /// `DEFAULT_MAX_RETRIES` is the default consensus parameter max_retries.
    pub const DEFAULT_MAX_RETRIES: u32 = 3;

    /// `DEFAULT_TIMEOUT` is the default consensus parameter timeout.
    pub const DEFAULT_TIMEOUT: u64 = 180;

    /// `DEFAULT_S_COST` is the default s_cost parameter value.
    pub const DEFAULT_S_COST: u32 = BalloonParams::DEFAULT_S_COST;

    /// `DEFAULT_T_COST` is the default t_cost parameter value.
    pub const DEFAULT_T_COST: u32 = BalloonParams::DEFAULT_T_COST;

    /// `DEFAULT_DELTA` is the default delta parameter value.
    pub const DEFAULT_DELTA: u32 = BalloonParams::DEFAULT_DELTA;

    /// `new` creates a new `ConsensusConfig`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        k: Option<u32>,
        alpha: Option<u32>,
        beta1: Option<u32>,
        beta2: Option<u32>,
        s_cost: Option<u32>,
        t_cost: Option<u32>,
        delta: Option<u32>,
        max_retries: Option<u32>,
        timeout: Option<u64>,
    ) -> Result<ConsensusConfig> {
        let k = Some(k.unwrap_or(Self::DEFAULT_K));

        let alpha = Some(alpha.unwrap_or(Self::DEFAULT_ALPHA));

        let beta1 = Some(beta1.unwrap_or(Self::DEFAULT_BETA1));

        let beta2 = Some(beta2.unwrap_or(Self::DEFAULT_BETA2));

        let s_cost = s_cost.unwrap_or(Self::DEFAULT_S_COST);

        let t_cost = t_cost.unwrap_or(Self::DEFAULT_T_COST);

        let delta = delta.unwrap_or(Self::DEFAULT_DELTA);

        let _ = BalloonParams::new(s_cost, t_cost, delta)?;

        let max_retries = Some(max_retries.unwrap_or(Self::DEFAULT_MAX_RETRIES));

        let timeout = Some(timeout.unwrap_or(Self::DEFAULT_TIMEOUT));

        let config = ConsensusConfig {
            k,
            alpha,
            beta1,
            beta2,
            s_cost: Some(s_cost),
            t_cost: Some(t_cost),
            delta: Some(delta),
            max_retries,
            timeout,
        };

        Ok(config)
    }

    /// `populate` populates the `None` fields in the `ConsensusConfig` when there are
    /// defaults.
    pub fn populate(&mut self) {
        if self.k.is_none() {
            self.k = Some(Self::DEFAULT_K);
        }

        if self.alpha.is_none() {
            self.alpha = Some(Self::DEFAULT_ALPHA);
        }

        if self.beta1.is_none() {
            self.beta1 = Some(Self::DEFAULT_BETA1);
        }

        if self.beta2.is_none() {
            self.beta2 = Some(Self::DEFAULT_BETA2);
        }

        if self.s_cost.is_none() {
            self.s_cost = Some(Self::DEFAULT_S_COST);
        }

        if self.t_cost.is_none() {
            self.t_cost = Some(Self::DEFAULT_T_COST);
        }

        if self.delta.is_none() {
            self.delta = Some(Self::DEFAULT_DELTA);
        }

        if self.max_retries.is_none() {
            self.max_retries = Some(Self::DEFAULT_MAX_RETRIES);
        }

        if self.timeout.is_none() {
            self.timeout = Some(Self::DEFAULT_TIMEOUT);
        }
    }

    /// `validate` validates the `ConsensusConfig`.
    pub fn validate(&self) -> Result<()> {
        let s_cost = self.s_cost.unwrap_or(Self::DEFAULT_S_COST);
        let t_cost = self.t_cost.unwrap_or(Self::DEFAULT_T_COST);
        let delta = self.delta.unwrap_or(Self::DEFAULT_DELTA);

        BalloonParams::new(s_cost, t_cost, delta)
            .map_err(|e| e.into())
            .map(|_| ())
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

    /// `to_toml` converts the `ConsensusConfig` into a TOML string.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string(self).map_err(|e| e.into())
    }

    /// `from_toml` converts a TOML string into an `ConsensusConfig`.
    pub fn from_toml(s: &str) -> Result<ConsensusConfig> {
        toml::from_str(s).map_err(|e| e.into())
    }
}

impl Default for ConsensusConfig {
    fn default() -> ConsensusConfig {
        let k = Some(ConsensusConfig::DEFAULT_K);
        let alpha = Some(ConsensusConfig::DEFAULT_ALPHA);
        let beta1 = Some(ConsensusConfig::DEFAULT_BETA1);
        let beta2 = Some(ConsensusConfig::DEFAULT_BETA2);
        let s_cost = Some(ConsensusConfig::DEFAULT_S_COST);
        let t_cost = Some(ConsensusConfig::DEFAULT_T_COST);
        let delta = Some(ConsensusConfig::DEFAULT_DELTA);
        let max_retries = Some(ConsensusConfig::DEFAULT_MAX_RETRIES);
        let timeout = Some(ConsensusConfig::DEFAULT_TIMEOUT);

        ConsensusConfig {
            k,
            alpha,
            beta1,
            beta2,
            s_cost,
            t_cost,
            delta,
            max_retries,
            timeout,
        }
    }
}

#[test]
fn test_consensus_config_new() {
    let invalid_s_cost = 0;
    let valid_s_cost = 1;
    let invalid_t_cost = 0;
    let valid_t_cost = 1;
    let invalid_delta = 0;
    let valid_delta = 4;

    let res = ConsensusConfig::new(
        None,
        None,
        None,
        None,
        Some(invalid_s_cost),
        None,
        None,
        None,
        None,
    );
    assert!(res.is_err());

    let res = ConsensusConfig::new(
        None,
        None,
        None,
        None,
        None,
        Some(invalid_t_cost),
        None,
        None,
        None,
    );
    assert!(res.is_err());

    let res = ConsensusConfig::new(
        None,
        None,
        None,
        None,
        None,
        None,
        Some(invalid_delta),
        None,
        None,
    );
    assert!(res.is_err());

    let res = ConsensusConfig::new(
        None,
        None,
        None,
        None,
        Some(valid_s_cost),
        Some(valid_t_cost),
        Some(valid_delta),
        None,
        None,
    );
    assert!(res.is_ok());
}

#[test]
fn test_consensus_config_validate() {
    let invalid_s_cost = 0;
    let invalid_t_cost = 0;
    let invalid_delta = 0;

    let mut config =
        ConsensusConfig::new(None, None, None, None, None, None, None, None, None).unwrap();

    let res = config.validate();
    assert!(res.is_ok());

    config.populate();

    let res = config.validate();
    assert!(res.is_ok());

    config.s_cost = Some(invalid_s_cost);

    let res = config.validate();
    assert!(res.is_err());

    config.s_cost = None;
    config.populate();

    config.t_cost = Some(invalid_t_cost);

    let res = config.validate();
    assert!(res.is_err());

    config.t_cost = None;
    config.populate();

    config.delta = Some(invalid_delta);

    let res = config.validate();
    assert!(res.is_err());
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

#[test]
fn test_consensus_config_serialize_toml() {
    let config_a = ConsensusConfig::default();

    let res = config_a.to_toml();
    assert!(res.is_ok());
    let toml = res.unwrap();

    let res = ConsensusConfig::from_toml(&toml);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}
