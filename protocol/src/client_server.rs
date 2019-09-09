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
        state.lock().unwrap().validate()?;
        logger.validate()?;

        let server = ProtocolClientServer {
            state,
            transport,
            logger,
        };

        Ok(server)
    }

    /// `validate` validates the `ProtocolClientServer`.
    pub fn validate(&self) -> Result<()> {
        self.state.lock().unwrap().validate()?;
        self.logger.validate().map_err(|e| e.into())
    }

    /// `serve` serves the main server operations.
    pub fn serve(&mut self) -> Result<()> {
        serve_client(self.state.clone(), self.transport.clone())
    }
}
