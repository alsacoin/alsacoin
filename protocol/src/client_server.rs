//! # Protocol Client Server
//!
//! `client_server` is the module containing the protocol client server type and functions.

use crate::network::serve_client;
use crate::result::{handle_result, Result};
use crate::state::ProtocolState;
use log::logger::Logger;
use network::traits::Network;
use std::sync::{Arc, Mutex};
use store::traits::Store;

/// `ProtocolClientServer` is the protocol client server type.
pub struct ProtocolClientServer<S, P, N>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    N: Network + Send + 'static,
{
    pub state: Arc<Mutex<ProtocolState<S, P>>>,
    pub network: Arc<Mutex<N>>,
    pub logger: Arc<Logger>,
}

impl<S, P, N> ProtocolClientServer<S, P, N>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    N: Network + Send + 'static,
{
    /// `new` creates a new `ProtocolClientServer`.
    pub fn new(
        state: Arc<Mutex<ProtocolState<S, P>>>,
        network: Arc<Mutex<N>>,
        logger: Arc<Logger>,
    ) -> Result<ProtocolClientServer<S, P, N>> {
        logger.log_info("Creating a new protocol client server")?;

        let res = state.lock().unwrap().validate();

        handle_result(logger.clone(), res, "Protocol client server creation error")?;

        let server = ProtocolClientServer {
            state,
            network,
            logger,
        };

        server
            .logger
            .log_info("New protocol client server created")?;

        Ok(server)
    }

    /// `validate` validates the `ProtocolClientServer`.
    pub fn validate(&self) -> Result<()> {
        self.logger
            .log_info("Validating the protocol client server")?;

        let res = self.state.lock().unwrap().validate();

        handle_result(
            self.logger.clone(),
            res,
            "Protocol client server validate error",
        )?;

        self.logger
            .log_info("Protocol client server validated")
            .map_err(|e| e.into())
    }

    /// `run` runs the `ProtocolClientServer`.
    pub fn run(&mut self) -> Result<()> {
        self.logger
            .log_info("Starting the protocol client server")?;

        let res = serve_client(
            self.state.clone(),
            self.network.clone(),
            self.logger.clone(),
        );

        handle_result(self.logger.clone(), res, "Protocol client server run error")?;

        self.logger
            .log_info("Protocol client server closed")
            .map_err(|e| e.into())
    }
}
