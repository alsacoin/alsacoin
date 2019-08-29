//! # Network Config
//!
//! `network_config` is the module containing the network configuration type and functions.

use crate::error::Error;
use crate::result::Result;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;

/// `NetworkConfig` is the type representing a network configuration.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub kind: Option<String>,
    pub address: Option<String>,
}

impl NetworkConfig {
    /// `VALID_KINDS` sets the valid network kinds.
    pub const VALID_KINDS: &'static [&'static str] = &["channels", "tcp"];

    /// `DEFAULT_KIND` is the default network kind.
    pub const DEFAULT_KIND: &'static str = "tcp";

    /// `new` creates a new `NetworkConfig`.
    pub fn new(kind: Option<String>, address: Option<String>) -> Result<NetworkConfig> {
        let kind = if let Some(kind) = kind {
            if !Self::VALID_KINDS.contains(&kind.as_str()) {
                let err = Error::InvalidKind;
                return Err(err);
            }

            kind
        } else {
            Self::DEFAULT_KIND.into()
        };

        let config = NetworkConfig {
            kind: Some(kind),
            address,
        };

        Ok(config)
    }

    /// `validate` validates the `NetworkConfig`.
    pub fn validate(&self) -> Result<()> {
        if let Some(ref kind) = self.kind {
            if !Self::VALID_KINDS.contains(&kind.as_str()) {
                let err = Error::InvalidKind;
                return Err(err);
            }
        }

        Ok(())
    }

    /// `to_bytes` converts the `NetworkConfig` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `NetworkConfig`.
    pub fn from_bytes(b: &[u8]) -> Result<NetworkConfig> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `NetworkConfig` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `NetworkConfig`.
    pub fn from_json(s: &str) -> Result<NetworkConfig> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

impl Default for NetworkConfig {
    fn default() -> NetworkConfig {
        let kind = Some(NetworkConfig::DEFAULT_KIND.into());
        let address = None;

        NetworkConfig { kind, address }
    }
}

#[test]
fn test_network_config_new() {}

#[test]
fn test_network_config_validate() {}

#[test]
fn test_network_config_serialize_bytes() {
    let config_a = NetworkConfig::default();

    let res = config_a.to_bytes();
    assert!(res.is_ok());
    let cbor = res.unwrap();

    let res = NetworkConfig::from_bytes(&cbor);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}

#[test]
fn test_network_config_serialize_json() {
    let config_a = NetworkConfig::default();

    let res = config_a.to_json();
    assert!(res.is_ok());
    let json = res.unwrap();

    let res = NetworkConfig::from_json(&json);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}
