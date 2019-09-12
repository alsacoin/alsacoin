//! # Status
//!
//! `status` contains the CLI status type and functions.

use crate::error::Error;
use crate::result::Result;
use crypto::random::Random;
use std::fmt;

/// `CliStatus` is the type of the CLI status.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u8)]
pub enum CliStatus {
    None,
    Starting,
    Running,
    Stopping,
}

impl CliStatus {
    /// Parses a `CliStatus` from a `&str`.
    pub fn parse(s: &str) -> Result<CliStatus> {
        match s {
            "none" => Ok(CliStatus::None),
            "starting" => Ok(CliStatus::Starting),
            "running" => Ok(CliStatus::Running),
            "stopping" => Ok(CliStatus::Stopping),
            _ => {
                let err = Error::InvalidCliStatus;
                Err(err)
            }
        }
    }

    /// `random` creates a new random `CliStatus`.
    pub fn random() -> Result<CliStatus> {
        let status_u8 = Random::u32_range(0, 3)? as u8;
        CliStatus::from_u8(status_u8)
    }

    /// `from_u8` converts an `u8` into a `CliStatus`.
    pub fn from_u8(n: u8) -> Result<CliStatus> {
        match n {
            0 => Ok(CliStatus::None),
            1 => Ok(CliStatus::Starting),
            2 => Ok(CliStatus::Running),
            3 => Ok(CliStatus::Stopping),
            _ => {
                let msg = "cannot parse into a CliStatus".into();
                let err = Error::Parse { msg };
                Err(err)
            }
        }
    }
}

impl fmt::Display for CliStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CliStatus::None => write!(f, "none"),
            CliStatus::Starting => write!(f, "starting"),
            CliStatus::Running => write!(f, "running"),
            CliStatus::Stopping => write!(f, "stopping"),
        }
    }
}

impl Default for CliStatus {
    fn default() -> CliStatus {
        CliStatus::None
    }
}

#[test]
fn test_cli_status_parse() {
    let valid_status_a = "running";

    let res = CliStatus::parse(valid_status_a);
    assert!(res.is_ok());

    let valid_status_b = res.unwrap();
    assert_eq!(valid_status_a, format!("{}", valid_status_b));

    let invalid_status = "status";

    let res = CliStatus::parse(invalid_status);
    assert!(res.is_err());
}

#[test]
fn test_cli_status_from_u8() {
    let status_nums = [0, 1, 2, 3, 4, 10, 127, 255];

    for num in &status_nums {
        let res = CliStatus::from_u8(*num);
        if *num < 4 {
            assert!(res.is_ok());
        } else {
            assert!(res.is_err());
        }
    }
}
