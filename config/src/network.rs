//! # Network Config
//!
//! `network` is the module containing the network configuration type and functions.

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
    pub consensus_address: Option<String>,
    pub miner_address: Option<String>,
    pub client_address: Option<String>,
}

impl NetworkConfig {
    /// `VALID_KINDS` sets the valid network kinds.
    pub const VALID_KINDS: &'static [&'static str] = &["channels", "tcp"];

    /// `DEFAULT_KIND` is the default network kind.
    pub const DEFAULT_KIND: &'static str = "tcp";

    /// `DEFAULT_CONSENSUS_ADDRESS` is the default consensus server address.
    pub const DEFAULT_CONSENSUS_ADDRESS: &'static str = "127.0.0.1:2019";

    /// `DEFAULT_MINER_ADDRESS` is the default miner address.
    pub const DEFAULT_MINER_ADDRESS: &'static str = "127.0.0.1:2020";

    /// `DEFAULT_CLIENT_ADDRESS` is the default client server address.
    pub const DEFAULT_CLIENT_ADDRESS: &'static str = "127.0.0.1:2021";

    /// `new` creates a new `NetworkConfig`.
    pub fn new(
        kind: Option<String>,
        consensus_address: Option<String>,
        miner_address: Option<String>,
        client_address: Option<String>,
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

        let same_addresses = consensus_address == miner_address
            || consensus_address == client_address
            || miner_address == client_address;

        let not_all_none =
            consensus_address.is_some() || miner_address.is_some() || client_address.is_some();

        if same_addresses && not_all_none {
            let err = Error::InvalidAddress;
            return Err(err);
        }

        let consensus_address = if consensus_address.is_none() {
            Some(Self::DEFAULT_CONSENSUS_ADDRESS.into())
        } else {
            None
        };

        let miner_address = if miner_address.is_none() {
            Some(Self::DEFAULT_MINER_ADDRESS.into())
        } else {
            None
        };

        let client_address = if client_address.is_none() {
            Some(Self::DEFAULT_CLIENT_ADDRESS.into())
        } else {
            None
        };

        let config = NetworkConfig {
            kind: Some(kind),
            consensus_address,
            miner_address,
            client_address,
        };

        Ok(config)
    }

    /// `populate` populates the `None` fields in the `NetworkConfig` when there are
    /// defaults.
    pub fn populate(&mut self) {
        if self.kind.is_none() {
            self.kind = Some(Self::DEFAULT_KIND.into());
        }

        if self.consensus_address.is_none() {
            self.consensus_address = Some(Self::DEFAULT_CONSENSUS_ADDRESS.into());
        }

        if self.miner_address.is_none() {
            self.miner_address = Some(Self::DEFAULT_MINER_ADDRESS.into());
        }

        if self.client_address.is_none() {
            self.client_address = Some(Self::DEFAULT_CLIENT_ADDRESS.into());
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

        let same_addresses = self.consensus_address == self.miner_address
            || self.consensus_address == self.client_address
            || self.miner_address == self.client_address;

        let not_all_none = self.consensus_address.is_some()
            || self.miner_address.is_some()
            || self.client_address.is_some();

        if same_addresses && not_all_none {
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
        let consensus_address = Some(NetworkConfig::DEFAULT_CONSENSUS_ADDRESS.into());
        let miner_address = Some(NetworkConfig::DEFAULT_MINER_ADDRESS.into());
        let client_address = Some(NetworkConfig::DEFAULT_CLIENT_ADDRESS.into());

        NetworkConfig {
            kind,
            consensus_address,
            miner_address,
            client_address,
        }
    }
}

#[test]
fn test_network_new() {
    let invalid_kind: String = "kind".into();
    let address = "address";

    let res = NetworkConfig::new(Some(invalid_kind.into()), None, None, None);
    assert!(res.is_err());

    let res = NetworkConfig::new(None, Some(address.into()), Some(address.into()), None);
    assert!(res.is_err());

    let res = NetworkConfig::new(None, Some(address.into()), None, Some(address.into()));
    assert!(res.is_err());

    let res = NetworkConfig::new(None, None, Some(address.into()), Some(address.into()));
    assert!(res.is_err());

    for kind in NetworkConfig::VALID_KINDS.iter().copied() {
        let res = NetworkConfig::new(Some(kind.into()), None, None, None);
        assert!(res.is_ok());
    }
}

#[test]
fn test_network_validate() {
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
fn test_network_serialize_bytes() {
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
fn test_network_serialize_json() {
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
fn test_network_serialize_toml() {
    let config_a = NetworkConfig::default();

    let res = config_a.to_toml();
    assert!(res.is_ok());
    let toml = res.unwrap();

    let res = NetworkConfig::from_toml(&toml);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}
