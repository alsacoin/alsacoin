//! # Error
//!
//! `error` contains the `consensus` crate `Error` type.

use std::fmt::Display;

#[derive(Debug, Display, Fail)]
pub enum Error {}
