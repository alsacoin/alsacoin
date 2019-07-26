//! # Utils
//!
//! `utils` contains utility functions used in the crate.

use crate::error::Error;
use crate::result::Result;
use regex::Regex;
use std::collections::HashMap;

/// Returns if the target string matches the regex pattern.
pub fn is_match(pattern: &str, target: &str) -> Result<bool> {
    let reg = Regex::new(pattern)?;
    Ok(reg.is_match(target))
}

/// Returns the regex captures obtained from the target string against the regex pattern.
pub fn captures(pattern: &str, target: &str) -> Result<HashMap<String, String>> {
    let reg = Regex::new(pattern)?;
    if !reg.is_match(target) {
        let err = Error::NoRegexMatch;
        return Err(err);
    }

    let mut res = HashMap::<String, String>::new();

    let _captures = reg.captures(target);

    if _captures.is_none() {
        return Ok(res);
    }

    let captures = _captures.unwrap();

    for cap_name in reg.capture_names() {
        if cap_name.is_some() {
            let key = cap_name.unwrap();
            let mut value = "";
            if let Some(cap_match) = captures.name(key) {
                value = cap_match.as_str();
            }
            res.insert(String::from(key), String::from(value));
        }
    }

    Ok(res)
}

#[test]
fn test_regex_is_match() {
    let email_pattern: &str =
        "(^(?P<username>[a-zA-Z0-9_.+-]+)@(?P<host>[a-zA-Z0-9-]+)\\.(?P<domain>[a-zA-Z0-9-.]+)$)";

    let valid_email = "test@example.com";

    let res = is_match(email_pattern, valid_email);
    assert!(res.is_ok());
    assert!(res.unwrap());

    let invalid_email = "test/@example.com";

    let res = is_match(email_pattern, invalid_email);
    assert!(res.is_ok());
    assert!(!res.unwrap())
}

#[test]
fn test_regex_captures() {
    let email_pattern: &str =
        "(^(?P<username>[a-zA-Z0-9_.+-]+)@(?P<host>[a-zA-Z0-9-]+)\\.(?P<domain>[a-zA-Z0-9-.]+)$)";

    let valid_email = "test@example.com";

    let res = captures(email_pattern, valid_email);
    assert!(res.is_ok());

    let capts = res.unwrap();

    let username_ = capts.get("username");
    assert!(username_.is_some());
    let username = username_.unwrap();
    assert_eq!(username, "test");

    let host_ = capts.get("host");
    assert!(host_.is_some());
    let host = host_.unwrap();
    assert_eq!(host, "example");

    let domain_ = capts.get("domain");
    assert!(domain_.is_some());
    let domain = domain_.unwrap();
    assert_eq!(domain, "com");

    let invalid_email = "test/@example.com";

    let res = captures(email_pattern, invalid_email);
    assert!(res.is_err());
}
