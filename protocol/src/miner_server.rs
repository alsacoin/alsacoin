//! # Protocol Miner Server
//!
//! `miner_server` is the module containing the protocol miner server type and functions.

use crate::common::handle_result;
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
        logger.log_info("Creating a new protocol miner server")?;

        let res = state.lock().unwrap().validate();

        handle_result(logger.clone(), res, "Protocol miner server creation error")?;

        let server = ProtocolMinerServer {
            state,
            transport,
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
            self.transport.clone(),
            self.logger.clone(),
        );

        handle_result(self.logger.clone(), res, "Protocol miner server run error")?;

        self.logger
            .log_info("Protocol miner server closed")
            .map_err(|e| e.into())
    }
}
