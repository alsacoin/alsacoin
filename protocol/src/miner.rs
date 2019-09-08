//! # Protocol Miner
//!
//! `miner` is the module containing the protocol miner type and functions.

use crate::network::serve_mining;
use crate::result::Result;
use crate::state::ProtocolState;
use log::logger::Logger;
use network::traits::Transport;
use std::sync::{Arc, Mutex};
use store::traits::Store;

/// `ProtocolMiner` is the protocol miner type.
pub struct ProtocolMiner<S, P, T>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    T: Transport + Send + 'static,
{
    pub state: Arc<Mutex<ProtocolState<S, P>>>,
    pub transport: Arc<Mutex<T>>,
    pub logger: Arc<Logger>,
}

impl<S, P, T> ProtocolMiner<S, P, T>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    T: Transport + Send + 'static,
{
    /// `new` creates a new `ProtocolMiner`.
    pub fn new(
        state: Arc<Mutex<ProtocolState<S, P>>>,
        transport: Arc<Mutex<T>>,
        logger: Arc<Logger>,
    ) -> Result<ProtocolMiner<S, P, T>> {
        state.lock().unwrap().validate()?;
        logger.validate()?;

        let miner = ProtocolMiner {
            state,
            transport,
            logger,
        };

        Ok(miner)
    }

    /// `validate` validates the `ProtocolMiner`.
    pub fn validate(&self) -> Result<()> {
        self.state.lock().unwrap().validate()?;
        self.logger.validate().map_err(|e| e.into())
    }

    /// `serve` serves the mining operations.
    pub fn serve(&mut self) -> Result<()> {
        serve_mining(self.state.clone(), self.transport.clone())
    }
}
