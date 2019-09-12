//! # Protocol Consensus Server
//!
//! `consensus_server` is the module containing the protocol consensus server type and functions.

use crate::network::serve_consensus;
use crate::result::{handle_result, Result};
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
        logger.log_info("Creating a new protocol consensus server")?;

        let res = state.lock().unwrap().validate();

        handle_result(
            logger.clone(),
            res,
            "Protocol consensus server creation error",
        )?;

        let server = ProtocolConsensusServer {
            state,
            transport,
            logger,
        };

        server
            .logger
            .log_info("New protocol consensus server created")?;

        Ok(server)
    }

    /// `validate` validates the `ProtocolConsensusServer`.
    pub fn validate(&self) -> Result<()> {
        self.logger
            .log_info("Validating the protocol consensus server")?;

        let res = self.state.lock().unwrap().validate();

        handle_result(
            self.logger.clone(),
            res,
            "Protocol consensus server validate error",
        )?;

        self.logger
            .log_info("Protocol consensus server validated")
            .map_err(|e| e.into())
    }

    /// `run` runs the `ProtocolConsensusServer`.
    pub fn run(&mut self) -> Result<()> {
        self.logger
            .log_info("Starting the protocol consensus server")?;

        let res = serve_consensus(
            self.state.clone(),
            self.transport.clone(),
            self.logger.clone(),
        );

        handle_result(
            self.logger.clone(),
            res,
            "Protocol consensus server run error",
        )?;

        self.logger
            .log_info("Protocol consensus server closed")
            .map_err(|e| e.into())
    }
}
