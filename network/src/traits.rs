//! # Traits
//!
//! `traits` contains Alsacoin's storage traits.

use crate::result::Result;
use models::node::Node;

/// `Transport` is the trait implemented by `Alsacoin` network transports.
pub trait Transport {
    /// `send` sends data to a `Node`.
    fn send(node: &Node, data: &[u8]) -> Result<()>;

    /// `recv` receives data from a `Node`.
    fn recv(node: &Node) -> Result<Vec<u8>>;
}
