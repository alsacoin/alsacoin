//! # Timestamp
//!
//! `timestamp` contains the timestamping types and functions.

use crate::error::Error;
use crate::result::Result;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The starting date time.
pub const MIN_DATETIME: &str = "2019-07-25T00:00:00Z";

/// The maximum accepted error noise of time measures. The internet is messy.
pub const MAX_TIMENOISE: i64 = 3_600;

/// A `Timestamp` is an integer representing the number of seconds elapsed since
/// the `Epoch` time (1970-01-01:00:00:00.0000...).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Serialize, Deserialize)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Creates a UTC unix `Timestamp` from a given date.
    pub fn new(
        year: u32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
    ) -> Result<Timestamp> {
        if month > 12 || month == 0 {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }

        if day > 31 {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }

        if hour > 23 {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }

        if min > 59 {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }

        if sec > 59 {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }

        let dt = Utc.ymd(year as i32, month, day).and_hms(hour, min, sec);

        let _timestamp = dt.timestamp();

        Ok(Timestamp(_timestamp))
    }

    /// Returns the minimum `Timestamp`.
    pub fn min_value() -> Timestamp {
        Timestamp::parse(MIN_DATETIME).unwrap()
    }

    /// Creates a `Timestamp` from a UTC date time string in rfc3339 format.
    /// e.g.: `2018-01-18T00:00:00Z`
    pub fn parse(s: &str) -> Result<Timestamp> {
        let dt = s.parse::<DateTime<Utc>>()?;

        let _timestamp = dt.timestamp();

        Ok(Timestamp(_timestamp))
    }

    /// Creates a `Timestamp` from a string.
    pub fn from_string(s: &str) -> Result<Timestamp> {
        Ok(Timestamp(i64::from_str_radix(s, 10)?))
    }

    /// Converts the `Timestamp` to string.
    pub fn to_string(self) -> String {
        format!("{:?}", self.0)
    }

    /// `from_i64` converts an `i64` into a `Timestamp`.
    pub fn from_i64(t: i64) -> Result<Timestamp> {
        let t = Timestamp(t);
        t.validate()?;

        Ok(t)
    }

    /// `to_i64` returns the `Timestamp` as an `i64`.
    pub fn to_i64(self) -> i64 {
        self.0
    }

    /// Returns the current time timestamp.
    pub fn now() -> Timestamp {
        Timestamp(Utc::now().timestamp())
    }

    /// Returns the `Timestamp` with the maximum time noise.
    pub fn with_noise(self) -> Timestamp {
        Timestamp(self.0 + MAX_TIMENOISE)
    }

    /// Returns the time difference between this `Timestamp` and an other.
    pub fn diff(self, other: Timestamp) -> i64 {
        self.0 - other.0
    }

    /// Validates the `Timestamp`.
    pub fn validate(self) -> Result<()> {
        if self < Timestamp::min_value() {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }

        if self > Timestamp::now().with_noise() {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }

        Ok(())
    }
}

impl Default for Timestamp {
    fn default() -> Timestamp {
        Timestamp::now()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[test]
fn test_timestamp_new() {
    use crypto::random::Random;

    for _ in 0..10 {
        let year = Random::u32().unwrap();
        let month = Random::u32().unwrap();
        let day = Random::u32().unwrap();
        let hour = Random::u32().unwrap();
        let min = Random::u32().unwrap();
        let sec = Random::u32().unwrap();

        let res = Timestamp::new(year, month, day, hour, min, sec);
        if year < 1970 || month > 12 || day > 31 || hour > 59 || min > 59 || sec > 59 {
            assert!(res.is_err());
        } else {
            assert!(res.is_ok());
        }
    }
}

#[test]
fn test_timestamp_parse() {
    let valid_date = "2012-12-12T00:00:00Z";
    let invalid_date = "2012-12-32T00:00:00Z";

    let res = Timestamp::parse(valid_date);
    assert!(res.is_ok());

    let res = Timestamp::parse(invalid_date);
    assert!(res.is_err())
}

#[test]
fn test_timestamp_to_string() {
    let date = "2012-12-12T00:00:00Z";

    let timestamp_a = Timestamp::parse(date).unwrap();
    let timestamp_str = timestamp_a.to_string();
    let timestamp_b = Timestamp::from_string(&timestamp_str).unwrap();

    assert_eq!(timestamp_a, timestamp_b)
}

#[test]
fn test_timestamp_i64() {
    let invalid_i64 = -1_000_000_000;
    let valid_i64 = Timestamp::now().to_i64();

    let res = Timestamp::from_i64(invalid_i64);
    assert!(res.is_err());

    let res = Timestamp::from_i64(valid_i64 + 60);
    assert!(res.is_ok());

    let timestamp = res.unwrap();
    let timestamp_i64 = timestamp.to_i64();

    assert_eq!(valid_i64, timestamp_i64 - 60);
}

#[test]
fn test_timestamp_now() {
    let timestamp = Timestamp::now();

    let res = timestamp.validate();
    assert!(res.is_ok());

    let noisy_timestamp = Timestamp::now().with_noise();

    let res = noisy_timestamp.validate();
    assert!(res.is_ok());
}

#[test]
fn test_timestamp_validate() {
    let date = "2012-12-12T00:00:00Z";
    let invalid_timestamp = Timestamp::parse(date).unwrap();
    let valid_timestamp = Timestamp::now();

    let res = invalid_timestamp.validate();
    assert!(res.is_err());

    let res = valid_timestamp.validate();
    assert!(res.is_ok());
}
