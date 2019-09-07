//! # Config
//!
//! `config` is the module containing the configuration type and functions.

use crate::consensus_config::ConsensusConfig;
use crate::error::Error;
use crate::log_config::LogConfig;
use crate::network_config::NetworkConfig;
use crate::pool_config::PoolConfig;
use crate::result::Result;
use crate::store_config::StoreConfig;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use toml;

/// `Config` is the type representing an Alsacoin configuration.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Config {
    pub stage: Option<String>,
    pub store_config: StoreConfig,
    pub pool_config: PoolConfig,
    pub network_config: NetworkConfig,
    pub log_config: LogConfig,
    pub consensus_config: ConsensusConfig,
}

impl Config {
    /// `VALID_STAGES` sets the valid stages.
    pub const VALID_STAGES: &'static [&'static str] = &["testing", "development", "production"];

    /// `DEFAULT_STAGE` is the default stage.
    pub const DEFAULT_STAGE: &'static str = "production";

    /// `new` creates a new `Config`.
    pub fn new(
        stage: Option<String>,
        store_conf: &StoreConfig,
        pool_conf: &PoolConfig,
        net_conf: &NetworkConfig,
        log_conf: &LogConfig,
        cons_conf: &ConsensusConfig,
    ) -> Result<Config> {
        let stage = if let Some(stage) = stage {
            if !Self::VALID_STAGES.contains(&stage.as_str()) {
                let err = Error::InvalidStage;
                return Err(err);
            }

            Some(stage)
        } else {
            Some(Self::DEFAULT_STAGE.into())
        };

        store_conf.validate()?;
        pool_conf.validate()?;
        net_conf.validate()?;
        log_conf.validate()?;
        cons_conf.validate()?;

        let conf = Config {
            stage,
            store_config: store_conf.to_owned(),
            pool_config: pool_conf.to_owned(),
            network_config: net_conf.to_owned(),
            log_config: log_conf.to_owned(),
            consensus_config: cons_conf.to_owned(),
        };

        Ok(conf)
    }

    /// `populate` populates the `None` fields in the `Config` when there are
    /// defaults.
    pub fn populate(&mut self) {
        if self.stage.is_none() {
            self.stage = Some(Self::DEFAULT_STAGE.into());
        }

        self.store_config.populate();
        self.pool_config.populate();
        self.network_config.populate();
        self.log_config.populate();
        self.consensus_config.populate();
    }

    /// `validate` validates the `Config`.
    pub fn validate(&self) -> Result<()> {
        if let Some(ref stage) = self.stage {
            if !Self::VALID_STAGES.contains(&stage.as_str()) {
                let err = Error::InvalidStage;
                return Err(err);
            }
        };

        self.store_config.validate()?;
        self.pool_config.validate()?;
        self.network_config.validate()?;
        self.log_config.validate()?;
        self.consensus_config.validate()?;

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
        let stage = Some(Config::DEFAULT_STAGE.into());
        let store_config = StoreConfig::default();
        let pool_config = PoolConfig::default();
        let network_config = NetworkConfig::default();
        let log_config = LogConfig::default();
        let consensus_config = ConsensusConfig::default();

        Config {
            stage,
            store_config,
            pool_config,
            network_config,
            log_config,
            consensus_config,
        }
    }
}

#[test]
fn test_config_new() {
    let invalid_kind = "kind";
    let invalid_stage = "stage";
    let invalid_level = "level";
    let invalid_s_cost = 0;

    let store_conf = StoreConfig::default();
    let pool_conf = PoolConfig::default();
    let net_conf = NetworkConfig::default();
    let log_conf = LogConfig::default();
    let cons_conf = ConsensusConfig::default();

    let mut invalid_store_conf = store_conf.clone();
    invalid_store_conf.kind = Some(invalid_kind.into());

    let mut invalid_pool_conf = pool_conf.clone();
    invalid_pool_conf.kind = Some(invalid_kind.into());

    let mut invalid_net_conf = net_conf.clone();
    invalid_net_conf.kind = Some(invalid_kind.into());

    let mut invalid_log_conf = log_conf.clone();
    invalid_log_conf.level = Some(invalid_level.into());

    let mut invalid_cons_conf = cons_conf.clone();
    invalid_cons_conf.s_cost = Some(invalid_s_cost);

    let res = Config::new(
        Some(invalid_stage.into()),
        &store_conf,
        &pool_conf,
        &net_conf,
        &log_conf,
        &cons_conf,
    );
    assert!(res.is_err());

    for stage in Config::VALID_STAGES.iter().copied() {
        let res = Config::new(
            Some(stage.into()),
            &invalid_store_conf,
            &pool_conf,
            &net_conf,
            &log_conf,
            &cons_conf,
        );
        assert!(res.is_err());

        let res = Config::new(
            Some(stage.into()),
            &store_conf,
            &invalid_pool_conf,
            &net_conf,
            &log_conf,
            &cons_conf,
        );
        assert!(res.is_err());

        let res = Config::new(
            Some(stage.into()),
            &store_conf,
            &pool_conf,
            &invalid_net_conf,
            &log_conf,
            &cons_conf,
        );
        assert!(res.is_err());

        let res = Config::new(
            Some(stage.into()),
            &store_conf,
            &pool_conf,
            &net_conf,
            &invalid_log_conf,
            &cons_conf,
        );
        assert!(res.is_err());

        let res = Config::new(
            Some(stage.into()),
            &store_conf,
            &pool_conf,
            &net_conf,
            &log_conf,
            &invalid_cons_conf,
        );
        assert!(res.is_err());

        let res = Config::new(
            Some(stage.into()),
            &store_conf,
            &pool_conf,
            &net_conf,
            &log_conf,
            &cons_conf,
        );
        assert!(res.is_ok());
    }
}

#[test]
fn test_config_validate() {
    let invalid_kind = "kind";
    let invalid_stage = "stage";
    let invalid_level = "level";
    let invalid_s_cost = 0;

    let store_conf = StoreConfig::default();
    let pool_conf = PoolConfig::default();
    let net_conf = NetworkConfig::default();
    let log_conf = LogConfig::default();

    let mut invalid_store_conf = StoreConfig::default();
    invalid_store_conf.kind = Some(invalid_kind.into());

    let mut invalid_pool_conf = PoolConfig::default();
    invalid_pool_conf.kind = Some(invalid_kind.into());

    let mut invalid_net_conf = NetworkConfig::default();
    invalid_net_conf.kind = Some(invalid_kind.into());

    let mut invalid_log_conf = LogConfig::default();
    invalid_log_conf.level = Some(invalid_level.into());

    let mut invalid_cons_conf = ConsensusConfig::default();
    invalid_cons_conf.s_cost = Some(invalid_s_cost);

    let mut config = Config::default();

    let res = config.validate();
    assert!(res.is_ok());

    config.stage = None;
    let res = config.validate();
    assert!(res.is_ok());

    config.populate();
    let res = config.validate();
    assert!(res.is_ok());

    config.stage = Some(invalid_stage.into());
    let res = config.validate();
    assert!(res.is_err());

    config.stage = None;

    config.store_config = invalid_store_conf;
    let res = config.validate();
    assert!(res.is_err());

    config.store_config = store_conf;

    config.pool_config = invalid_pool_conf;
    let res = config.validate();
    assert!(res.is_err());

    config.pool_config = pool_conf;

    config.network_config = invalid_net_conf;
    let res = config.validate();
    assert!(res.is_err());

    config.network_config = net_conf;

    config.log_config = invalid_log_conf;
    let res = config.validate();
    assert!(res.is_err());

    config.log_config = log_conf;

    config.consensus_config = invalid_cons_conf;
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
