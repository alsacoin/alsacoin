//! # Protocol Client Server
//!
//! `client_server` is the module containing the protocol client server type and functions.

use crate::network::serve_client;
use crate::result::Result;
use crate::state::ProtocolState;
use log::logger::Logger;
use network::traits::Transport;
use std::sync::{Arc, Mutex};
use store::traits::Store;

/// `ProtocolClientServer` is the protocol client server type.
pub struct ProtocolClientServer<S, P, T>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    T: Transport + Send + 'static,
{
    pub state: Arc<Mutex<ProtocolState<S, P>>>,
    pub transport: Arc<Mutex<T>>,
    pub logger: Arc<Logger>,
}

impl<S, P, T> ProtocolClientServer<S, P, T>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    T: Transport + Send + 'static,
{
    /// `new` creates a new `ProtocolClientServer`.
    pub fn new(
        state: Arc<Mutex<ProtocolState<S, P>>>,
        transport: Arc<Mutex<T>>,
        logger: Arc<Logger>,
    ) -> Result<ProtocolClientServer<S, P, T>> {
        logger.log_info("Creating a new protocol client server")?;

        let res = state.lock().unwrap().validate();

        match res {
            Ok(_) => {}
            Err(err) => {
                let msg = format!("Protocol client server creation error: {}", err);
                logger.log_critical(&msg)?;
            }
        }

        let server = ProtocolClientServer {
            state,
            transport,
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

        match res {
            Ok(_) => self
                .logger
                .log_info("Protocol client server validated")
                .map_err(|e| e.into()),
            Err(err) => {
                let msg = format!("Protocol client server validation error: {}", err);
                self.logger.log_critical(&msg).map_err(|e| e.into())
            }
        }
    }

    /// `run` runs the `ProtocolClientServer`.
    pub fn run(&mut self) -> Result<()> {
        self.logger
            .log_info("Starting the protocol client server")?;

        let res = serve_client(
            self.state.clone(),
            self.transport.clone(),
            self.logger.clone(),
        );

        match res {
            Ok(_) => self
                .logger
                .log_info("Protocol client server closed")
                .map_err(|e| e.into()),
            Err(err) => {
                let msg = format!("Protocol client server closed with error: {}", err);
                self.logger.log_critical(&msg).map_err(|e| e.into())
            }
        }
    }
}
