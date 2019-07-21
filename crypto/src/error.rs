//! # Error
//!
//! `error` contains the `crypto` crate `Error` type.

use std::fmt::Display;

#[derive(Debug, Display, Fail)]
pub enum Error {}


