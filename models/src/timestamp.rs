//! # Timestamp
//!
//! `timestamp` contains the timestamping types and functions.

use chrono::{DateTime, TimeZone, Utc};
use serde::{Serialize, Deserialize};
use crate::error::Error;
use crate::result::Result;
use std::fmt;

/// The starting date time.
pub const MIN_DATETIME: &str = "2019-07-25T00:00:00Z";

/// The maximum accepted error noise of time measures. The internet is messy.
pub const MAX_TIMENOISE: i64 = 3_600;

/// A `Timestamp` is an integer representing the number of seconds elapsed since
/// the `Epoch` time (1970-01-01:00:00:00.0000...).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Creates a UTC unix `Timestamp` from a given date.
    pub fn from_date(year: i32,
                     month: u32,
                     day: u32,
                     hours: u32,
                     mins: u32,
                     secs: u32) -> Result<Timestamp>
    {
        if day > 31 {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }
        
        if hours > 24 {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }
        
        if mins > 60 {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }
        
        if secs > 60 {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }
        
        let dt = Utc.ymd(year, month, day)
            .and_hms(mins, hours, secs);

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
fn timestamp_from_date_succ() {
    let year = 2012;
    let month = 12;
    let day = 12;
    let hours = 12;
    let mins = 12;
    let secs = 12;

    let res = Timestamp::from_date(year, month, day,
                                   hours, mins, secs);
    assert!(res.is_ok())
}

#[test]
fn timestamp_from_date_fail() {
    let year = 2012;
    let month = 12;
    let day = 32;
    let hours = 12;
    let mins = 12;
    let secs = 12;
    
    let res = Timestamp::from_date(year, month, day,
                                   hours, mins, secs);
    assert!(res.is_err())
}

#[test]
fn timestamp_parse_succ() {
    let date = "2012-12-12T00:00:00Z";
    
    let res = Timestamp::parse(date);
    assert!(res.is_ok())
}

#[test]
fn timestamp_parse_fail() {
    let date = "2012-12-32T00:00:00Z";
    
    let res = Timestamp::parse(date);
    assert!(res.is_err())
}

#[test]
fn timestamp_to_string_succ() {
    let date = "2012-12-12T00:00:00Z";
    
    let timestamp_a = Timestamp::parse(date).unwrap();
    let timestamp_str = timestamp_a.to_string();
    let timestamp_b = Timestamp::from_string(&timestamp_str).unwrap();
    
    assert_eq!(timestamp_a, timestamp_b)
}

#[test]
fn timestamp_to_string_fail() {
    let date = "2012-12-12T00:00:00Z";
    
    let timestamp_a = Timestamp::parse(date).unwrap();
    let mut timestamp_str = timestamp_a.to_string();
    timestamp_str.pop();
    let timestamp_b = Timestamp::from_string(&timestamp_str).unwrap();
    
    assert_ne!(timestamp_a, timestamp_b)
}

#[test]
fn timestamp_validate_succ() {
    let timestamp = Timestamp::now();
    
    let res = timestamp.validate();
    assert!(res.is_ok())
}

#[test]
fn timestamp_validate_fail() {
    let date = "2012-12-12T00:00:00Z";
    let timestamp = Timestamp::parse(date).unwrap();
    
    let res = timestamp.validate();
    assert!(res.is_err())
}
