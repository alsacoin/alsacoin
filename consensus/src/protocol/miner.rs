//! # Protocol Miner
//!
//! `miner` is the module containing the protocol miner type and functions.

use crate::protocol::network::serve_mining;
use crate::protocol::ProtocolState;
use crate::result::Result;
use network::traits::Transport;
use std::sync::{Arc, Mutex};
use store::traits::Store;

/// `ProtocolMiner` is the protocol miner type.
pub struct ProtocolMiner<S: Store, P: Store, T: Transport> {
    pub state: Arc<Mutex<ProtocolState<S, P>>>,
    pub transport: Arc<Mutex<T>>,
}

impl<S: Store, P: Store, T: Transport> ProtocolMiner<S, P, T> {
    /// `new` creates a new `ProtocolMiner`.
    pub fn new(
        state: Arc<Mutex<ProtocolState<S, P>>>,
        transport: Arc<Mutex<T>>,
    ) -> Result<ProtocolMiner<S, P, T>> {
        state.lock().unwrap().validate()?;

        let miner = ProtocolMiner { state, transport };

        Ok(miner)
    }

    /// `validate` validates the `ProtocolMiner`.
    pub fn validate(&self) -> Result<()> {
        self.state.lock().unwrap().validate()
    }

    /// `serve` serves the mining operations.
    pub fn serve(&mut self) -> Result<()> {
        serve_mining(self.state.clone(), self.transport.clone())
    }
}
