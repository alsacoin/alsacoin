//! # Store Config
//!
//! `store_config` is the module containing the store configuration type and functions.

use crate::error::Error;
use crate::result::Result;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;

/// `StoreConfig` is the type representing a store configuration.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
pub struct StoreConfig {
    pub kind: Option<String>,
    pub path: Option<String>,
    pub max_value_size: Option<u32>,
    pub max_size: Option<u32>,
}

impl StoreConfig {
    /// `VALID_KINDS` sets the valid store kinds.
    pub const VALID_KINDS: &'static [&'static str] = &["temporary", "persistent"];

    /// `DEFAULT_KIND` is the default store kind.
    pub const DEFAULT_KIND: &'static str = "persistent";

    /// `DEFAULT_MAX_VALUE_SIZE` is the default store max_value_size.
    pub const DEFAULT_MAX_VALUE_SIZE: u32 = 1 << 10;

    /// `DEFAULT_MAX_SIZE` is the default store max_size.
    pub const DEFAULT_MAX_SIZE: u32 = 1 << 30;

    /// `new` creates a new `StoreConfig`.
    pub fn new(
        kind: Option<String>,
        path: Option<String>,
        max_value_size: Option<u32>,
        max_size: Option<u32>,
    ) -> Result<StoreConfig> {
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

        let config = StoreConfig {
            kind: Some(kind),
            path,
            max_value_size: Some(max_value_size),
            max_size: Some(max_size),
        };

        Ok(config)
    }

    /// `validate` validates the `StoreConfig`.
    pub fn validate(&self) -> Result<()> {
        if let Some(ref kind) = self.kind {
            if !Self::VALID_KINDS.contains(&kind.as_str()) {
                let err = Error::InvalidKind;
                return Err(err);
            }
        }

        Ok(())
    }

    /// `to_bytes` converts the `StoreConfig` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `StoreConfig`.
    pub fn from_bytes(b: &[u8]) -> Result<StoreConfig> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `StoreConfig` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `StoreConfig`.
    pub fn from_json(s: &str) -> Result<StoreConfig> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}
