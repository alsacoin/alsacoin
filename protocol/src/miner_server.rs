//! # Protocol Miner Server
//!
//! `miner_server` is the module containing the protocol miner server type and functions.

use crate::network::serve_mining;
use crate::result::{handle_result, Result};
use crate::state::ProtocolState;
use log::logger::Logger;
use network::traits::Network;
use std::sync::{Arc, Mutex};
use store::traits::Store;

/// `ProtocolMinerServer` is the protocol miner server type.
pub struct ProtocolMinerServer<S, P, N>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    N: Network + Send + 'static,
{
    pub state: Arc<Mutex<ProtocolState<S, P>>>,
    pub network: Arc<Mutex<N>>,
    pub logger: Arc<Logger>,
}

impl<S, P, N> ProtocolMinerServer<S, P, N>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    N: Network + Send + 'static,
{
    /// `new` creates a new `ProtocolMinerServer`.
    pub fn new(
        state: Arc<Mutex<ProtocolState<S, P>>>,
        network: Arc<Mutex<N>>,
        logger: Arc<Logger>,
    ) -> Result<ProtocolMinerServer<S, P, N>> {
        logger.log_info("Creating a new protocol miner server")?;

        let res = state.lock().unwrap().validate();

        handle_result(logger.clone(), res, "Protocol miner server creation error")?;

        let server = ProtocolMinerServer {
            state,
            network,
            logger,
        };

        server
            .logger
            .log_info("New protocol miner server created")?;

        Ok(server)
    }

    /// `validate` validates the `ProtocolMinerServer`.
    pub fn validate(&self) -> Result<()> {
        self.logger
            .log_info("Validating the protocol miner server")?;

        let res = self.state.lock().unwrap().validate();

        handle_result(
            self.logger.clone(),
            res,
            "Protocol miner server validate error",
        )?;

        self.logger
            .log_info("Protocol miner server validated")
            .map_err(|e| e.into())
    }

    /// `run` runs the `ProtocolMinerServer`.
    pub fn run(&mut self) -> Result<()> {
        self.logger.log_info("Starting the protocol miner server")?;

        let res = serve_mining(
            self.state.clone(),
            self.network.clone(),
            self.logger.clone(),
        );

        handle_result(self.logger.clone(), res, "Protocol miner server run error")?;

        self.logger
            .log_info("Protocol miner server closed")
            .map_err(|e| e.into())
    }
}
