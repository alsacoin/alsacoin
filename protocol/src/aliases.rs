//! # Aliases
//!
//! `aliases` contains the main alias types of the crate.

use crate::client;
use crate::client_server;
use crate::consensus_server;
use crate::miner_server;
use crate::state;
use network::backend::TcpNetwork;
use store::backend::UnQLiteStore;

pub type ProtocolState = state::ProtocolState<UnQLiteStore, UnQLiteStore>;

pub type ProtocolClient = client::ProtocolClient<UnQLiteStore, UnQLiteStore, TcpNetwork>;

pub type ProtocolClientServer =
    client_server::ProtocolClientServer<UnQLiteStore, UnQLiteStore, TcpNetwork>;

pub type ProtocolConsensusServer =
    consensus_server::ProtocolConsensusServer<UnQLiteStore, UnQLiteStore, TcpNetwork>;

pub type ProtocolMinerServer =
    miner_server::ProtocolMinerServer<UnQLiteStore, UnQLiteStore, TcpNetwork>;
