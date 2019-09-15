//! # Pool Config
//!
//! `pool` is the module containing the pool configuration type and functions.

use crate::result::Result;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use toml;

/// `PoolConfig` is the type representing a pool configuration.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct PoolConfig {
    pub max_value_size: Option<u32>,
    pub max_size: Option<u32>,
}

impl PoolConfig {
    /// `DEFAULT_MAX_VALUE_SIZE` is the default pool max_value_size.
    pub const DEFAULT_MAX_VALUE_SIZE: u32 = 1 << 30;

    /// `DEFAULT_MAX_SIZE` is the default pool max_size.
    pub const DEFAULT_MAX_SIZE: u32 = 1 << 30;

    /// `new` creates a new `PoolConfig`.
    pub fn new(max_value_size: Option<u32>, max_size: Option<u32>) -> PoolConfig {
        let max_value_size = max_value_size.unwrap_or(Self::DEFAULT_MAX_VALUE_SIZE);

        let max_size = max_size.unwrap_or(Self::DEFAULT_MAX_SIZE);

        PoolConfig {
            max_value_size: Some(max_value_size),
            max_size: Some(max_size),
        }
    }

    /// `populate` populates the `None` fields in the `PoolConfig` when there are
    /// defaults.
    pub fn populate(&mut self) {
        if self.max_value_size.is_none() {
            self.max_value_size = Some(Self::DEFAULT_MAX_VALUE_SIZE);
        }

        if self.max_size.is_none() {
            self.max_size = Some(Self::DEFAULT_MAX_SIZE);
        }
    }

    /// `to_bytes` converts the `PoolConfig` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `PoolConfig`.
    pub fn from_bytes(b: &[u8]) -> Result<PoolConfig> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `PoolConfig` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `PoolConfig`.
    pub fn from_json(s: &str) -> Result<PoolConfig> {
        serde_json::from_str(s).map_err(|e| e.into())
    }

    /// `to_toml` converts the `PoolConfig` into a TOML string.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string(self).map_err(|e| e.into())
    }

    /// `from_toml` converts a TOML string into an `PoolConfig`.
    pub fn from_toml(s: &str) -> Result<PoolConfig> {
        toml::from_str(s).map_err(|e| e.into())
    }
}

impl Default for PoolConfig {
    fn default() -> PoolConfig {
        let max_value_size = Some(PoolConfig::DEFAULT_MAX_VALUE_SIZE);
        let max_size = Some(PoolConfig::DEFAULT_MAX_SIZE);

        PoolConfig {
            max_value_size,
            max_size,
        }
    }
}

#[test]
fn test_pool_serialize_bytes() {
    let config_a = PoolConfig::default();

    let res = config_a.to_bytes();
    assert!(res.is_ok());
    let cbor = res.unwrap();

    let res = PoolConfig::from_bytes(&cbor);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}

#[test]
fn test_pool_serialize_json() {
    let config_a = PoolConfig::default();

    let res = config_a.to_json();
    assert!(res.is_ok());
    let json = res.unwrap();

    let res = PoolConfig::from_json(&json);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}

#[test]
fn test_pool_serialize_toml() {
    let config_a = PoolConfig::default();

    let res = config_a.to_toml();
    assert!(res.is_ok());
    let toml = res.unwrap();

    let res = PoolConfig::from_toml(&toml);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}
