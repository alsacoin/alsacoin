//! # Network Config
//!
//! `network_config` is the module containing the network configuration type and functions.

use crate::error::Error;
use crate::result::Result;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use toml;

/// `NetworkConfig` is the type representing a network configuration.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub kind: Option<String>,
    pub server_address: Option<String>,
    pub miner_address: Option<String>,
}

impl NetworkConfig {
    /// `VALID_KINDS` sets the valid network kinds.
    pub const VALID_KINDS: &'static [&'static str] = &["channels", "tcp"];

    /// `DEFAULT_KIND` is the default network kind.
    pub const DEFAULT_KIND: &'static str = "tcp";

    /// `DEFAULT_SERVER_ADDRESS` is the default server address.
    pub const DEFAULT_SERVER_ADDRESS: &'static str = "127.0.0.1:2019";

    /// `DEFAULT_MINER_ADDRESS` is the default miner address.
    pub const DEFAULT_MINER_ADDRESS: &'static str = "127.0.0.1:2020";

    /// `new` creates a new `NetworkConfig`.
    pub fn new(
        kind: Option<String>,
        server_address: Option<String>,
        miner_address: Option<String>,
    ) -> Result<NetworkConfig> {
        let kind = if let Some(kind) = kind {
            if !Self::VALID_KINDS.contains(&kind.as_str()) {
                let err = Error::InvalidKind;
                return Err(err);
            }

            kind
        } else {
            Self::DEFAULT_KIND.into()
        };

        if let Some(ref saddress) = server_address {
            if let Some(ref maddress) = miner_address {
                if saddress == maddress {
                    let err = Error::InvalidAddress;
                    return Err(err);
                }
            }
        }

        let server_address = if server_address.is_none() {
            Some(Self::DEFAULT_SERVER_ADDRESS.into())
        } else {
            None
        };

        let miner_address = if miner_address.is_none() {
            Some(Self::DEFAULT_MINER_ADDRESS.into())
        } else {
            None
        };

        let config = NetworkConfig {
            kind: Some(kind),
            server_address,
            miner_address,
        };

        Ok(config)
    }

    /// `populate` populates the `None` fields in the `NetworkConfig` when there are
    /// defaults.
    pub fn populate(&mut self) {
        if self.kind.is_none() {
            self.kind = Some(Self::DEFAULT_KIND.into());
        }

        if self.server_address.is_none() {
            self.server_address = Some(Self::DEFAULT_SERVER_ADDRESS.into());
        }

        if self.miner_address.is_none() {
            self.miner_address = Some(Self::DEFAULT_MINER_ADDRESS.into());
        }
    }

    /// `validate` validates the `NetworkConfig`.
    pub fn validate(&self) -> Result<()> {
        if let Some(ref kind) = self.kind {
            if !Self::VALID_KINDS.contains(&kind.as_str()) {
                let err = Error::InvalidKind;
                return Err(err);
            }
        }

        if self.server_address.is_some()
            && self.miner_address.is_some()
            && self.server_address == self.miner_address
        {
            let err = Error::InvalidAddress;
            return Err(err);
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

    /// `to_toml` converts the `NetworkConfig` into a TOML string.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string(self).map_err(|e| e.into())
    }

    /// `from_toml` converts a TOML string into an `NetworkConfig`.
    pub fn from_toml(s: &str) -> Result<NetworkConfig> {
        toml::from_str(s).map_err(|e| e.into())
    }
}

impl Default for NetworkConfig {
    fn default() -> NetworkConfig {
        let kind = Some(NetworkConfig::DEFAULT_KIND.into());
        let server_address = Some(NetworkConfig::DEFAULT_SERVER_ADDRESS.into());
        let miner_address = Some(NetworkConfig::DEFAULT_MINER_ADDRESS.into());

        NetworkConfig {
            kind,
            server_address,
            miner_address,
        }
    }
}

#[test]
fn test_network_config_new() {
    let invalid_kind: String = "kind".into();
    let address = "address";

    let res = NetworkConfig::new(Some(invalid_kind.into()), None, None);
    assert!(res.is_err());

    let res = NetworkConfig::new(None, Some(address.into()), Some(address.into()));
    assert!(res.is_err());

    for kind in NetworkConfig::VALID_KINDS.iter().copied() {
        let res = NetworkConfig::new(Some(kind.into()), None, None);
        assert!(res.is_ok());
    }
}

#[test]
fn test_network_config_validate() {
    let mut config = NetworkConfig::default();

    let res = config.validate();
    assert!(res.is_ok());

    config.kind = None;
    let res = config.validate();
    assert!(res.is_ok());

    config.populate();
    let res = config.validate();
    assert!(res.is_ok());

    config.kind = Some("".into());
    let res = config.validate();
    assert!(res.is_err());
}

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

#[test]
fn test_network_config_serialize_toml() {
    let config_a = NetworkConfig::default();

    let res = config_a.to_toml();
    assert!(res.is_ok());
    let toml = res.unwrap();

    let res = NetworkConfig::from_toml(&toml);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}
