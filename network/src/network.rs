//! # Network
//!
//! `network` is the module containing the network type and functions.

use crate::backend::TcpNetwork;
use crate::error::Error;
use crate::result::Result;
use config::network::NetworkConfig;

/// `NetworkFactory` is the factory for network types.
pub struct NetworkFactory {}

impl NetworkFactory {
    /// `create` creates a new network from the configs.
    pub fn create(config: &NetworkConfig) -> Result<TcpNetwork> {
        config.validate()?;

        let mut config = config.clone();
        config.populate();

        match config.kind.unwrap().as_str() {
            "consensus" => {
                let addr = config.consensus_address.clone().unwrap();
                TcpNetwork::new(&addr)
            }
            "miner" => {
                let addr = config.miner_address.clone().unwrap();
                TcpNetwork::new(&addr)
            }
            "client" => {
                let addr = config.client_address.clone().unwrap();
                TcpNetwork::new(&addr)
            }
            _ => {
                let err = Error::InvalidKind;
                Err(err)
            }
        }
    }
}
