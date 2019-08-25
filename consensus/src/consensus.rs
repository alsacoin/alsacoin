//! # Consensus
//!
//! `consensus` is the module containing the type implementing the Avalanche consensus.

use models::consensus_params::ConsensusParams;
use models::consensus_state::ConsensusState;
use network::traits::Transport;
use store::traits::Store;

/// `Consensus` is the type encapsulating the Avalanche Consensus algorithm.
pub struct Consensus<S: Store, P: Store, T: Transport> {
    _params: ConsensusParams,
    _state: ConsensusState,
    _store: S,
    _pool: P,
    _transport: T,
}
