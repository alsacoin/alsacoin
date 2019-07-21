//! # Error
//!
//! `error` contains the `store` crate `Error` type.

use std::fmt::Display;

#[derive(Debug, Display, Fail)]
pub enum Error {}
