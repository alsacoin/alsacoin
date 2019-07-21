//! # Error
//!
//! `error` contains the `config` crate `Error` type.

use std::fmt::Display;

#[derive(Debug, Display, Fail)]
pub enum Error {}

