//! # Consensus
//!
//! `consensus` is the module containing the type implementing the Avalanche consensus.

use models::consensus_state::ConsensusState;
use network::traits::Transport;
use store::traits::Store;

/// `Consensus` is the type encapsulating the Avalanche Consensus algorithm.
pub struct Consensus<S: Store, P: Store, N: Transport> {
    _state: ConsensusState,
    _store: S,
    _pool: P,
    _network: N,
}
