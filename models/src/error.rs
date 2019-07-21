//! # Error
//!
//! `error` contains the `models` crate `Error` type.

use std::fmt::Display;

#[derive(Debug, Display, Fail)]
pub enum Error {}

