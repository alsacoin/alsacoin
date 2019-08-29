//! # Config
//!
//! `config` is the module containing the configuration type and functions.

use crate::consensus_config::ConsensusConfig;
use crate::network_config::NetworkConfig;
use crate::pool_config::PoolConfig;
use crate::result::Result;
use crate::store_config::StoreConfig;
use models::stage::Stage;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;

/// `Config` is the type representing an Alsacoin configuration.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Config {
    pub stage: Stage,
    pub store_config: StoreConfig,
    pub pool_config: PoolConfig,
    pub network_config: NetworkConfig,
    pub consensus_config: ConsensusConfig,
}

impl Config {
    /// `new` creates a new `Config`.
    pub fn new(
        stage: Stage,
        store_conf: &StoreConfig,
        pool_conf: &PoolConfig,
        net_conf: &NetworkConfig,
        cons_conf: &ConsensusConfig,
    ) -> Result<Config> {
        store_conf.validate()?;
        pool_conf.validate()?;
        net_conf.validate()?;

        let conf = Config {
            stage,
            store_config: store_conf.to_owned(),
            pool_config: pool_conf.to_owned(),
            network_config: net_conf.to_owned(),
            consensus_config: cons_conf.to_owned(),
        };

        Ok(conf)
    }

    /// `validate` validates the `Config`.
    pub fn validate(&self) -> Result<()> {
        self.store_config.validate()?;
        self.pool_config.validate()?;
        self.network_config.validate()?;

        Ok(())
    }

    /// `to_bytes` converts the `Config` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `Config`.
    pub fn from_bytes(b: &[u8]) -> Result<Config> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `Config` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `Config`.
    pub fn from_json(s: &str) -> Result<Config> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

impl Default for Config {
    fn default() -> Config {
        let stage = Stage::default();
        let store_config = StoreConfig::default();
        let pool_config = PoolConfig::default();
        let network_config = NetworkConfig::default();
        let consensus_config = ConsensusConfig::default();

        Config {
            stage,
            store_config,
            pool_config,
            network_config,
            consensus_config,
        }
    }
}
