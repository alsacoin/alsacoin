//! # Traits
//!
//! `traits` contains Alsacoin's storage traits.

use crate::result::Result;
use models::node::Node;

/// `Transport` is the trait implemented by `Alsacoin` network transports.
pub trait Transport {
    /// `local_address` returns the local `Node` address.
    fn local_address(&self) -> Result<Vec<u8>>;

    /// `send` sends data to a `Node`.
    fn send(&mut self, node: &Node, data: &[u8]) -> Result<()>;

    /// `recv` receives data from a `Node`.
    fn recv(&mut self) -> Result<(Node, Vec<u8>)>;
}
