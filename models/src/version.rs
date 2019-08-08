//! # Version
//!
//! `version` contains the Semver version type and functions.

use crate::error::Error;
use crate::result::Result;
use crate::utils;
use crypto::random::Random;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use std::cmp::Ordering;
use std::fmt;

/// Current Semver version of the library.
pub const VERSION: &str = "0.1.0";

/// Regex pattern of a numeric version.
pub const NUMERIC_VERSION: &str = "^[0-9]*$";

/// Regex pattern of a prerelease version.
pub const PRERELEASE_VERSION: &str = "^[A-Za-z-]*$";

/// Regex pattern of a buildmeta version.
pub const BUILDMETA_VERSION: &str = "^[0-9A-Za-z-]*$";

/// Regex pattern of a Semver version.
pub const SEMVER_VERSION: &str = "^(?P<major>[0-9]*).(?P<minor>[0-9]*).(?P<patch>[0-9]*)(-(?P<prerelease>[A-Za-z-]+))?(\\+(?P<buildmeta>[0-9A-Za-z-]+))?$";

/// Type used to represent a Semver version.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Version {
    /// Semver version major. Used for API breaking changes.
    pub major: u32,
    /// Semver version minor. Used for backward-compatible API changes.
    pub minor: u32,
    /// Semver version patch. Used for changes not affecting the API.
    pub patch: u32,
    /// Semver version prerelease. Used in prereleases. Optional.
    pub prerelease: String,
    /// Semver version buildmeta. Build information. Optional.
    pub buildmeta: String,
}

impl Version {
    /// Creates a new Semver version.
    pub fn new(major: u32, minor: u32, patch: u32, pre: &str, build: &str) -> Result<Version> {
        Self::validate_prerelease(pre)?;

        Self::validate_buildmeta(build)?;

        let prerelease = String::from(pre);
        let buildmeta = String::from(build);

        let ver = Version {
            major,
            minor,
            patch,
            prerelease,
            buildmeta,
        };

        Ok(ver)
    }

    /// `random` creates a new random `Version`.
    pub fn random() -> Result<Version> {
        let major = Random::u32()?;
        let minor = Random::u32()?;
        let patch = Random::u32()?;
        let pre = ""; // TODO: de-lame
        let build = ""; // TODO: de-lame

        Version::new(major, minor, patch, pre, build)
    }

    /// Validate a Semver numeric version (major, minor or patch).
    pub fn validate_numeric(num: &str) -> Result<()> {
        if utils::is_match(NUMERIC_VERSION, num).unwrap() {
            Ok(())
        } else {
            let err = Error::InvalidVersion;
            Err(err)
        }
    }

    /// Validate a Semver prerelease version.
    pub fn validate_prerelease(pre: &str) -> Result<()> {
        if utils::is_match(PRERELEASE_VERSION, pre).unwrap() {
            Ok(())
        } else {
            let err = Error::InvalidVersion;
            Err(err)
        }
    }

    /// Validate a buildmeta prerelease version.
    pub fn validate_buildmeta(build: &str) -> Result<()> {
        if utils::is_match(BUILDMETA_VERSION, build).unwrap() {
            Ok(())
        } else {
            let err = Error::InvalidVersion;
            Err(err)
        }
    }

    /// Validate a semver version.
    pub fn validate_semver(sv: &str) -> Result<()> {
        if utils::is_match(SEMVER_VERSION, sv).unwrap() {
            Ok(())
        } else {
            let err = Error::InvalidVersion;
            Err(err)
        }
    }

    /// Parse a string as a `Version`.
    pub fn parse(s: &str) -> Result<Version> {
        let matches = utils::captures(SEMVER_VERSION, s)?;

        let _major = matches.get("major").unwrap();
        let major = u32::from_str_radix(_major, 10)?;

        let _minor = matches.get("minor").unwrap();
        let minor = u32::from_str_radix(_minor, 10)?;

        let _patch = matches.get("patch").unwrap();
        let patch = u32::from_str_radix(_patch, 10)?;

        let _prerelease = matches.get("prerelease").unwrap();
        let prerelease = _prerelease.to_owned();
        let _buildmeta = matches.get("buildmeta").unwrap();
        let buildmeta = _buildmeta.to_owned();

        let ver = Version {
            major,
            minor,
            patch,
            prerelease,
            buildmeta,
        };

        Ok(ver)
    }

    /// Stringify the `Version`.
    #[allow(clippy::inherent_to_string_shadow_display)]
    pub fn to_string(&self) -> String {
        let mut res = String::new();

        res.push_str(&format!("{}", self.major));
        res.push_str(&format!(".{}", self.minor));
        res.push_str(&format!(".{}", self.patch));

        if !self.prerelease.is_empty() {
            res.push('-');
            res.push_str(&self.prerelease);
        }

        if !self.buildmeta.is_empty() {
            res.push('+');
            res.push_str(&self.buildmeta);
        }

        res
    }

    fn compare_numeric(n: u32, other: u32) -> Ordering {
        n.cmp(&other)
    }

    fn compare_prerelease(a: &str, b: &str) -> Ordering {
        if a.is_empty() {
            if b.is_empty() {
                return Ordering::Equal;
            }

            return Ordering::Greater;
        }

        if b.is_empty() {
            return Ordering::Less;
        }

        a.cmp(&b)
    }

    fn compare(&self, other: &Version) -> Ordering {
        let mut res = Self::compare_numeric(self.major, other.major);
        if res != Ordering::Equal {
            return res;
        }

        res = Self::compare_numeric(self.minor, other.minor);
        if res != Ordering::Equal {
            return res;
        }

        res = Self::compare_numeric(self.patch, other.patch);
        if res != Ordering::Equal {
            return res;
        }

        Self::compare_prerelease(&self.prerelease, &other.prerelease)
    }

    /// Returns if this `Version` is compatible to an other.
    pub fn is_compatible(&self, other: &Version) -> Result<bool> {
        self.validate()?;
        other.validate()?;

        let compatible = self.major == other.major;
        Ok(compatible)
    }

    /// Validates the `Version`.
    pub fn validate(&self) -> Result<()> {
        Self::validate_prerelease(&self.prerelease)?;

        Self::validate_buildmeta(&self.buildmeta)?;

        Ok(())
    }

    /// `to_bytes` converts the `Version` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `Version`.
    pub fn from_bytes(b: &[u8]) -> Result<Version> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `Version` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `Version`.
    pub fn from_json(s: &str) -> Result<Version> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

impl Default for Version {
    fn default() -> Self {
        Version::parse(VERSION).unwrap()
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

impl Eq for Version {}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        Some(self.compare(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        self.compare(other)
    }
}

#[test]
fn test_version_new() {
    let res = Version::new(1, 2, 3, "alpha", "build-01");
    assert!(res.is_ok());

    let res = Version::new(1, 2, 3, "///", "build-02");
    assert!(res.is_err());

    let res = Version::new(1, 2, 3, "beta", "àòlè@*)");
    assert!(res.is_err());
}

#[test]
fn test_version_default() {
    let version = Version::default();
    assert!(version.validate().is_ok());

    assert!(&version.to_string() == VERSION);
}

#[test]
fn test_version_validate_numeric() {
    let valid_numeric = "12345";

    let res = Version::validate_numeric(valid_numeric);
    assert!(res.is_ok());

    let invalid_numeric_a = "1234.9";
    let res = Version::validate_numeric(invalid_numeric_a);
    assert!(res.is_err());

    let invalid_numeric_b = "sdf8873439hf-pewrfjhdsjgvbcru";
    let res = Version::validate_numeric(invalid_numeric_b);
    assert!(res.is_err());
}

#[test]
fn test_version_validate_prerelease() {
    let valid_prerelease = "abc-DEF-ghj-";

    let res = Version::validate_prerelease(valid_prerelease);
    assert!(res.is_ok());

    let invalid_prerelease_a = "1234";
    let res = Version::validate_prerelease(invalid_prerelease_a);
    assert!(res.is_err());

    let invalid_prerelease_b = "!£$%";
    let res = Version::validate_prerelease(invalid_prerelease_b);
    assert!(res.is_err());
}

#[test]
fn test_version_validate_buildmeta() {
    let valid_buildmeta = "123-abc-DEF-";

    let res = Version::validate_buildmeta(valid_buildmeta);
    assert!(res.is_ok());

    let invalid_buildmeta = "&/(.";
    let res = Version::validate_buildmeta(invalid_buildmeta);
    assert!(res.is_err());
}

#[test]
fn test_version_validate_semver() {
    let valid_semver = "1.10.1947-abcd-EFG+1A-bc-2";

    let res = Version::validate_semver(valid_semver);
    assert!(res.is_ok());

    let invalid_semver_a = "1.10.194a";
    let res = Version::validate_semver(invalid_semver_a);
    assert!(res.is_err());

    let invalid_semver_b = "1.10.1947-";
    let res = Version::validate_semver(invalid_semver_b);
    assert!(res.is_err());
}

#[test]
fn test_version_parse() {
    let valid_version = "1.10.1947-abcd-EFG+1A-bc-2";

    let res = Version::parse(valid_version);
    assert!(res.is_ok());

    let invalid_version_a = "1.10.1947+";

    let res = Version::parse(invalid_version_a);
    assert!(res.is_err());

    let invalid_version_b = "a.10.1947";

    let res = Version::parse(invalid_version_b);
    assert!(res.is_err());

    let invalid_version_c = "1.b.1947";

    let res = Version::parse(invalid_version_c);
    assert!(res.is_err());

    let invalid_version_d = "1.10.c";

    let res = Version::parse(invalid_version_d);
    assert!(res.is_err());

    let invalid_version_e = "a.b.c";
    let res = Version::parse(invalid_version_e);
    assert!(res.is_err());
}

#[test]
fn test_version_to_string() {
    let valid_version = "1.10.1947-abcd-EFG+1A-bc-2";

    let version_a = Version::parse(valid_version).unwrap();
    let version_a_str = version_a.to_string();

    let version_b = Version::parse(&version_a_str).unwrap();
    assert_eq!(version_a, version_b);
}

#[test]
fn test_version_format() {
    let valid_version = "1.10.1947-abcd-EFG+1A-bc-2";

    let version_a = Version::parse(valid_version).unwrap();
    let version_a_str = format!("{}", version_a);

    let version_b = Version::parse(&version_a_str).unwrap();
    assert_eq!(version_a, version_b);
}

#[test]
fn test_version_ord() {
    let version_a = Version::parse("0.1.0").unwrap();
    let version_b = Version::parse("0.2.0").unwrap();
    let version_c = Version::parse("1.0.0").unwrap();
    let version_d = Version::parse("0.0.6").unwrap();
    let version_e = Version::parse("0.0.6-alpha").unwrap();
    let version_f = Version::parse("0.0.6-beta").unwrap();
    let version_g = Version::parse("0.0.6-beta+abuild").unwrap();

    assert!(version_a < version_b);
    assert!(version_a < version_c);
    assert!(version_b < version_c);
    assert!(version_d < version_a);
    assert!(version_d < version_b);
    assert!(version_d < version_c);
    assert!(version_e < version_d);
    assert!(version_e < version_f);
    assert!(version_f < version_d);
    assert!(version_g == version_f);
}

#[test]
fn test_version_is_compatible() {
    let version_a = Version::parse("1.0.2-alpha").unwrap();
    let version_b = Version::parse("1.0.2-beta").unwrap();
    let version_c = Version::parse("0.0.1+build-1947").unwrap();
    let mut invalid_version = Version::default();
    invalid_version.buildmeta = "....".into();

    let res = version_a.is_compatible(&version_b);
    assert!(res.is_ok());
    assert!(res.unwrap());

    let res = version_b.is_compatible(&version_c);
    assert!(res.is_ok());
    assert!(!res.unwrap());

    let res = version_c.is_compatible(&invalid_version);
    assert!(res.is_err());
}

#[test]
fn test_version_serialize_bytes() {
    for _ in 0..10 {
        let version_a = Version::random().unwrap();

        let res = version_a.to_bytes();
        assert!(res.is_ok());
        let cbor = res.unwrap();

        let res = Version::from_bytes(&cbor);
        assert!(res.is_ok());
        let version_b = res.unwrap();

        assert_eq!(version_a, version_b)
    }
}

#[test]
fn test_version_serialize_json() {
    for _ in 0..10 {
        let version_a = Version::random().unwrap();

        let res = version_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = Version::from_json(&json);
        assert!(res.is_ok());
        let version_b = res.unwrap();

        assert_eq!(version_a, version_b)
    }
}
