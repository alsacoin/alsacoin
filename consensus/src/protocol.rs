//! # Protocol
//!
//! `protocol` is the module containing the type implementing the Avalanche Consensus Protocol.

use models::consensus_params::ConsensusParams;
use models::consensus_state::ConsensusState;
use network::traits::Transport;
use store::traits::Store;

/// `Protocol` is the type encapsulating the Avalanche Consensus Protocol.
pub struct Protocol<S: Store, P: Store, T: Transport> {
    _params: ConsensusParams,
    _state: ConsensusState,
    _store: S,
    _pool: P,
    _transport: T,
}
