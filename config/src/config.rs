//! # Config
//!
//! `config` is the module containing the configuration type and functions.

use crate::consensus::ConsensusConfig;
use crate::log::LogConfig;
use crate::network::NetworkConfig;
use crate::pool::PoolConfig;
use crate::result::Result;
use crate::store::StoreConfig;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use toml;

/// `Config` is the type representing an Alsacoin configuration.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Config {
    pub store: StoreConfig,
    pub pool: PoolConfig,
    pub network: NetworkConfig,
    pub log: LogConfig,
    pub consensus: ConsensusConfig,
}

impl Config {
    /// `new` creates a new `Config`.
    pub fn new(
        store_conf: &StoreConfig,
        pool_conf: &PoolConfig,
        net_conf: &NetworkConfig,
        log_conf: &LogConfig,
        cons_conf: &ConsensusConfig,
    ) -> Result<Config> {
        store_conf.validate()?;
        net_conf.validate()?;
        log_conf.validate()?;
        cons_conf.validate()?;

        let conf = Config {
            store: store_conf.to_owned(),
            pool: pool_conf.to_owned(),
            network: net_conf.to_owned(),
            log: log_conf.to_owned(),
            consensus: cons_conf.to_owned(),
        };

        Ok(conf)
    }

    /// `populate` populates the `None` fields in the `Config` when there are
    /// defaults.
    pub fn populate(&mut self) {
        self.store.populate();
        self.pool.populate();
        self.network.populate();
        self.log.populate();
        self.consensus.populate();
    }

    /// `validate` validates the `Config`.
    pub fn validate(&self) -> Result<()> {
        self.store.validate()?;
        self.network.validate()?;
        self.log.validate()?;
        self.consensus.validate()?;

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

    /// `to_toml` converts the `Config` into a TOML string.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string(self).map_err(|e| e.into())
    }

    /// `from_toml` converts a TOML string into an `Config`.
    pub fn from_toml(s: &str) -> Result<Config> {
        toml::from_str(s).map_err(|e| e.into())
    }
}

impl Default for Config {
    fn default() -> Config {
        let store = StoreConfig::default();
        let pool = PoolConfig::default();
        let network = NetworkConfig::default();
        let log = LogConfig::default();
        let consensus = ConsensusConfig::default();

        Config {
            store,
            pool,
            network,
            log,
            consensus,
        }
    }
}

#[test]
fn test_config_new() {
    let invalid_kind = "kind";
    let invalid_level = "level";
    let invalid_s_cost = 0;

    let store_conf = StoreConfig::default();
    let pool_conf = PoolConfig::default();
    let net_conf = NetworkConfig::default();
    let log_conf = LogConfig::default();
    let cons_conf = ConsensusConfig::default();

    let mut invalid_store_conf = store_conf.clone();
    invalid_store_conf.kind = Some(invalid_kind.into());

    let mut invalid_net_conf = net_conf.clone();
    invalid_net_conf.kind = Some(invalid_kind.into());

    let mut invalid_log_conf = log_conf.clone();
    invalid_log_conf.level = Some(invalid_level.into());

    let mut invalid_cons_conf = cons_conf.clone();
    invalid_cons_conf.s_cost = Some(invalid_s_cost);

    let res = Config::new(&store_conf, &pool_conf, &net_conf, &log_conf, &cons_conf);
    assert!(res.is_ok());

    let res = Config::new(
        &invalid_store_conf,
        &pool_conf,
        &net_conf,
        &log_conf,
        &cons_conf,
    );
    assert!(res.is_err());

    let res = Config::new(
        &store_conf,
        &pool_conf,
        &invalid_net_conf,
        &log_conf,
        &cons_conf,
    );
    assert!(res.is_err());

    let res = Config::new(
        &store_conf,
        &pool_conf,
        &net_conf,
        &invalid_log_conf,
        &cons_conf,
    );
    assert!(res.is_err());

    let res = Config::new(
        &store_conf,
        &pool_conf,
        &net_conf,
        &log_conf,
        &invalid_cons_conf,
    );
    assert!(res.is_err());
}

#[test]
fn test_config_validate() {
    let invalid_kind = "kind";
    let invalid_level = "level";
    let invalid_s_cost = 0;

    let store_conf = StoreConfig::default();
    let pool_conf = PoolConfig::default();
    let net_conf = NetworkConfig::default();
    let log_conf = LogConfig::default();

    let mut invalid_store_conf = StoreConfig::default();
    invalid_store_conf.kind = Some(invalid_kind.into());

    let mut invalid_net_conf = NetworkConfig::default();
    invalid_net_conf.kind = Some(invalid_kind.into());

    let mut invalid_log_conf = LogConfig::default();
    invalid_log_conf.level = Some(invalid_level.into());

    let mut invalid_cons_conf = ConsensusConfig::default();
    invalid_cons_conf.s_cost = Some(invalid_s_cost);

    let mut config = Config::default();

    let res = config.validate();
    assert!(res.is_ok());

    config.populate();
    let res = config.validate();
    assert!(res.is_ok());

    config.store = invalid_store_conf;
    let res = config.validate();
    assert!(res.is_err());

    config.store = store_conf;

    config.network = invalid_net_conf;
    let res = config.validate();
    assert!(res.is_err());

    config.network = net_conf;

    config.log = invalid_log_conf;
    let res = config.validate();
    assert!(res.is_err());

    config.log = log_conf;

    config.consensus = invalid_cons_conf;
    let res = config.validate();
    assert!(res.is_err());
}

#[test]
fn test_config_serialize_bytes() {
    let config_a = Config::default();

    let res = config_a.to_bytes();
    assert!(res.is_ok());
    let cbor = res.unwrap();

    let res = Config::from_bytes(&cbor);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}

#[test]
fn test_config_serialize_json() {
    let config_a = Config::default();

    let res = config_a.to_json();
    assert!(res.is_ok());
    let json = res.unwrap();

    let res = Config::from_json(&json);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}

#[test]
fn test_config_serialize_toml() {
    let config_a = Config::default();

    let res = config_a.to_toml();
    assert!(res.is_ok());
    let toml = res.unwrap();

    let res = Config::from_toml(&toml);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}
