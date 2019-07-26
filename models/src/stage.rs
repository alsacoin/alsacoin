//! # Stage
//!
//! `stage` is the module containing the network stage type and functions.

use crate::error::Error;
use crate::result::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Enum representing the distributed ledger stage (development, testing or production).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Stage {
    /// Development stage.
    Development,
    /// Testing stage.
    Testing,
    /// Production stage.
    Production,
}

impl Stage {
    /// Parses a `Stage` from a `&str`.
    pub fn parse(s: &str) -> Result<Stage> {
        match s {
            "development" => Ok(Stage::Development),
            "testing" => Ok(Stage::Testing),
            "production" => Ok(Stage::Production),
            _ => {
                let err = Error::InvalidStage;
                Err(err)
            }
        }
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Stage::Development => write!(f, "development"),
            Stage::Testing => write!(f, "testing"),
            Stage::Production => write!(f, "production"),
        }
    }
}

impl Default for Stage {
    fn default() -> Stage {
        Stage::Development
    }
}

#[test]
fn test_stage_parse() {
    let valid_stage_a = "testing";

    let res = Stage::parse(valid_stage_a);
    assert!(res.is_ok());

    let valid_stage_b = res.unwrap();
    assert_eq!(valid_stage_a, format!("{}", valid_stage_b));

    let invalid_stage = "test";

    let res = Stage::parse(invalid_stage);
    assert!(res.is_err());
}
