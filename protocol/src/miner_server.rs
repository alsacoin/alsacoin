//! # Protocol Miner Server
//!
//! `miner_server` is the module containing the protocol miner server type and functions.

use crate::network::serve_mining;
use crate::result::Result;
use crate::state::ProtocolState;
use log::logger::Logger;
use network::traits::Transport;
use std::sync::{Arc, Mutex};
use store::traits::Store;

/// `ProtocolMinerServer` is the protocol miner server type.
pub struct ProtocolMinerServer<S, P, T>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    T: Transport + Send + 'static,
{
    pub state: Arc<Mutex<ProtocolState<S, P>>>,
    pub transport: Arc<Mutex<T>>,
    pub logger: Arc<Logger>,
}

impl<S, P, T> ProtocolMinerServer<S, P, T>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    T: Transport + Send + 'static,
{
    /// `new` creates a new `ProtocolMinerServer`.
    pub fn new(
        state: Arc<Mutex<ProtocolState<S, P>>>,
        transport: Arc<Mutex<T>>,
        logger: Arc<Logger>,
    ) -> Result<ProtocolMinerServer<S, P, T>> {
        logger.validate()?;
        let res = state.lock().unwrap().validate();

        match res {
            Ok(_) => {}
            Err(err) => {
                let msg = format!("{}", err);
                logger.log_critical(&msg)?;
            }
        }

        let server = ProtocolMinerServer {
            state,
            transport,
            logger,
        };

        Ok(server)
    }

    /// `validate` validates the `ProtocolMinerServer`.
    pub fn validate(&self) -> Result<()> {
        self.logger.validate()?;
        let res = self.state.lock().unwrap().validate();

        match res {
            Ok(_) => Ok(()),
            Err(err) => {
                let msg = format!("{}", err);
                self.logger.log_critical(&msg).map_err(|e| e.into())
            }
        }
    }

    /// `serve` serves the mining operations.
    pub fn serve(&mut self) -> Result<()> {
        let res = serve_mining(self.state.clone(), self.transport.clone());

        match res {
            Ok(_) => Ok(()),
            Err(err) => {
                let msg = format!("{}", err);
                self.logger.log_critical(&msg).map_err(|e| e.into())
            }
        }
    }
}
