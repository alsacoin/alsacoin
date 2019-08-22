//! # Traits
//!
//! `traits` contains Alsacoin's storage traits.

use crate::message::Message;
use crate::result::Result;
use std::ops::FnMut;

/// `Transport` is the trait implemented by `Alsacoin` network transports.
pub trait Transport {
    /// `local_address` returns the local `Node` address.
    fn local_address(&self) -> Result<Vec<u8>>;

    /// `send` sends data to a `Node`.
    fn send(&mut self, address: &[u8], data: &[u8]) -> Result<()>;

    /// `recv` receives data from a `Node`.
    fn recv(&mut self) -> Result<Message>;

    /// `serve` execs a given function on incoming `Message`s.
    fn serve<F: FnMut(Message) -> Result<()>>(&mut self, handler: F) -> Result<()>;
}
