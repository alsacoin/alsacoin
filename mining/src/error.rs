//! # Error
//!
//! `error` contains the `mining` crate `Error` type.

use std::fmt::Display;

#[derive(Debug, Display, Fail)]
pub enum Error {}

