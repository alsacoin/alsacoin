//! # Network
//!
//! `network` is the module containing the network type and functions.

use crate::error::Error;
use crate::node::TcpNode;
use crate::result::Result;
use crate::traits::Transport;
use config::network::NetworkConfig;

/// `NetworkFactory` is the factory for network types.
pub struct NetworkFactory {}

impl NetworkFactory {
    /// `create` creates a new network from the configs.
    pub fn create(config: &NetworkConfig) -> Result<Box<dyn Transport>> {
        config.validate()?;

        let mut config = config.clone();
        config.populate();

        match config.kind.unwrap().as_str() {
            "consensus" => {
                let addr = config.consensus_address.clone().unwrap();
                let network = TcpNode::new(&addr)?;

                Ok(Box::new(network))
            }
            "miner" => {
                let addr = config.miner_address.clone().unwrap();
                let network = TcpNode::new(&addr)?;

                Ok(Box::new(network))
            }
            "client" => {
                let addr = config.client_address.clone().unwrap();
                let network = TcpNode::new(&addr)?;

                Ok(Box::new(network))
            }
            _ => {
                let err = Error::InvalidKind;
                return Err(err);
            }
        }
    }
}
