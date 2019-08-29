//! # Pool Config
//!
//! `pool_config` is the module containing the pool configuration type and functions.

use crate::error::Error;
use crate::result::Result;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;

/// `PoolConfig` is the type representing a pool configuration.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct PoolConfig {
    pub kind: Option<String>,
    pub max_value_size: Option<u32>,
    pub max_size: Option<u32>,
}

impl PoolConfig {
    /// `VALID_KINDS` sets the valid pool kinds.
    pub const VALID_KINDS: &'static [&'static str] = &["btree_map", "unqlite"];

    /// `DEFAULT_KIND` is the default pool kind.
    pub const DEFAULT_KIND: &'static str = "unqlite";

    /// `DEFAULT_MAX_VALUE_SIZE` is the default pool max_value_size.
    pub const DEFAULT_MAX_VALUE_SIZE: u32 = 1 << 10;

    /// `DEFAULT_MAX_SIZE` is the default pool max_size.
    pub const DEFAULT_MAX_SIZE: u32 = 1 << 30;

    /// `new` creates a new `PoolConfig`.
    pub fn new(
        kind: Option<String>,
        max_value_size: Option<u32>,
        max_size: Option<u32>,
    ) -> Result<PoolConfig> {
        let kind = if let Some(kind) = kind {
            if !Self::VALID_KINDS.contains(&kind.as_str()) {
                let err = Error::InvalidKind;
                return Err(err);
            }

            kind
        } else {
            Self::DEFAULT_KIND.into()
        };

        let max_value_size = max_value_size.unwrap_or(Self::DEFAULT_MAX_VALUE_SIZE);

        let max_size = max_size.unwrap_or(Self::DEFAULT_MAX_SIZE);

        let config = PoolConfig {
            kind: Some(kind),
            max_value_size: Some(max_value_size),
            max_size: Some(max_size),
        };

        Ok(config)
    }

    /// `validate` validates the `PoolConfig`.
    pub fn validate(&self) -> Result<()> {
        if let Some(ref kind) = self.kind {
            if !Self::VALID_KINDS.contains(&kind.as_str()) {
                let err = Error::InvalidKind;
                return Err(err);
            }
        }

        Ok(())
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
}

impl Default for PoolConfig {
    fn default() -> PoolConfig {
        let kind = Some(PoolConfig::DEFAULT_KIND.into());
        let max_value_size = Some(PoolConfig::DEFAULT_MAX_VALUE_SIZE);
        let max_size = Some(PoolConfig::DEFAULT_MAX_SIZE);

        PoolConfig {
            kind,
            max_value_size,
            max_size,
        }
    }
}
