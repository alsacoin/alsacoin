//! # Protocol Consensus Server
//!
//! `consensus_server` is the module containing the protocol consensus server type and functions.

use crate::network::serve_consensus;
use crate::result::Result;
use crate::state::ProtocolState;
use log::logger::Logger;
use network::traits::Transport;
use std::sync::{Arc, Mutex};
use store::traits::Store;

/// `ProtocolConsensusServer` is the protocol consensus server type.
pub struct ProtocolConsensusServer<S, P, T>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    T: Transport + Send + 'static,
{
    pub state: Arc<Mutex<ProtocolState<S, P>>>,
    pub transport: Arc<Mutex<T>>,
    pub logger: Arc<Logger>,
}

impl<S, P, T> ProtocolConsensusServer<S, P, T>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    T: Transport + Send + 'static,
{
    /// `new` creates a new `ProtocolConsensusServer`.
    pub fn new(
        state: Arc<Mutex<ProtocolState<S, P>>>,
        transport: Arc<Mutex<T>>,
        logger: Arc<Logger>,
    ) -> Result<ProtocolConsensusServer<S, P, T>> {
        state.lock().unwrap().validate()?;
        logger.validate()?;

        let server = ProtocolConsensusServer {
            state,
            transport,
            logger,
        };

        Ok(server)
    }

    /// `validate` validates the `ProtocolConsensusServer`.
    pub fn validate(&self) -> Result<()> {
        self.state.lock().unwrap().validate()?;
        self.logger.validate().map_err(|e| e.into())
    }

    /// `serve` serves the main server operations.
    pub fn serve(&mut self) -> Result<()> {
        serve_consensus(self.state.clone(), self.transport.clone())
    }
}
