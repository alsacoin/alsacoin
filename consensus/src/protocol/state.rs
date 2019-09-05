//! # Protocol State
//!
//! `state` is the module containing the protocol state type and functions.

use crate::error::Error;
use crate::result::Result;
use config::consensus_config::ConsensusConfig;
use crypto::hash::Digest;
use models::account::Account;
use models::conflict_set::ConflictSet;
use models::consensus_state::ConsensusState;
use models::error::Error as ModelsError;
use models::node::Node;
use models::stage::Stage;
use models::traits::Storable;
use models::transaction::Transaction;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use store::traits::Store;

/// `ProtocolState` is the protocol state type.
#[allow(dead_code)]
pub struct ProtocolState<S: Store, P: Store> {
    stage: Stage,
    address: Vec<u8>,
    config: ConsensusConfig,
    state: ConsensusState,
    eve_account: Account,
    eve_transaction: Transaction,
    seed: BTreeSet<Node>,
    store: Arc<Mutex<S>>,
    pool: Arc<Mutex<P>>,
}

impl<S: Store, P: Store> ProtocolState<S, P> {
    /// `new` creates a new `ProtocolState` instance.
    /// The method is equivalent to the "Init" procedure in
    /// the Avalanche paper.
    pub fn new(
        address: &[u8],
        config: &ConsensusConfig,
        state: &ConsensusState,
        eve_account: &Account,
        seed: &BTreeSet<&[u8]>,
        store: Arc<Mutex<S>>,
        pool: Arc<Mutex<P>>,
    ) -> Result<ProtocolState<S, P>> {
        config.validate()?;
        state.validate()?;
        eve_account.validate()?;

        if !eve_account.is_eve()? {
            let err = Error::InvalidAccount;
            return Err(err);
        }

        let stage = state.stage;

        if eve_account.stage != stage {
            let err = Error::InvalidStage;
            return Err(err);
        }

        let mut eve_transaction = Transaction::new_eve(stage, &eve_account.address)?;
        eve_transaction.mine()?;

        Account::create(
            &mut *store.lock().unwrap(),
            stage,
            &eve_account.address,
            &eve_account,
        )?;

        Transaction::create(
            &mut *store.lock().unwrap(),
            stage,
            &eve_transaction.id,
            &eve_transaction,
        )?;

        let mut seed_nodes = BTreeSet::new();

        for address in seed {
            let node = Node::new(stage, address);
            Node::create(&mut *store.lock().unwrap(), stage, &node.id, &node)?;
            seed_nodes.insert(node);
        }

        let state = ProtocolState {
            stage,
            address: address.to_owned(),
            config: config.to_owned(),
            state: state.to_owned(),
            eve_account: eve_account.to_owned(),
            eve_transaction: eve_transaction.to_owned(),
            seed: seed_nodes,
            store,
            pool,
        };

        Ok(state)
    }

    /// `set_consensus_config` sets a new `ConsensusConfig` in the `ProtocolState`.
    pub fn set_consensus_config(&mut self, config: &ConsensusConfig) -> Result<()> {
        config.validate()?;

        self.config = config.to_owned();

        Ok(())
    }

    /// `set_consensus_state` sets a new `ConsensusState` in the `ProtocolState`.
    pub fn set_consensus_state(&mut self, state: &ConsensusState) -> Result<()> {
        state.validate()?;

        self.state = state.to_owned();

        Ok(())
    }

    /// `get_known_ancestors` returns a `Transaction` known ancestors.
    pub fn get_known_ancestors(&self, tx_id: &Digest) -> Result<BTreeSet<Digest>> {
        let tx = match Transaction::get(&*self.pool.lock().unwrap(), self.stage, tx_id) {
            Ok(tx) => Ok(tx),
            Err(ModelsError::NotFound) => {
                Transaction::get(&*self.store.lock().unwrap(), self.stage, tx_id)
            }
            Err(e) => Err(e),
        }?;

        tx.validate()?;

        let ancestors: BTreeSet<Digest> = tx
            .ancestors()?
            .iter()
            .filter(|id| self.state.lookup_known_transaction(&id))
            .copied()
            .collect();

        Ok(ancestors)
    }

    /// `get_unknown_ancestors` returns the unknown ancestors of a `Transactions`.
    pub fn get_unknown_ancestors(&self, tx_id: &Digest) -> Result<BTreeSet<Digest>> {
        let tx = match Transaction::get(&*self.pool.lock().unwrap(), self.stage, tx_id) {
            Ok(tx) => Ok(tx),
            Err(ModelsError::NotFound) => {
                Transaction::get(&*self.store.lock().unwrap(), self.stage, tx_id)
            }
            Err(e) => Err(e),
        }?;

        tx.validate()?;

        let ancestors: BTreeSet<Digest> = tx
            .ancestors()?
            .iter()
            .filter(|id| !self.state.lookup_known_transaction(&id))
            .copied()
            .collect();

        Ok(ancestors)
    }

    /// `get_transaction_conflict_set` returns a `Transaction` `ConflictSet`.
    pub fn get_transaction_conflict_set(&self, tx_id: &Digest) -> Result<ConflictSet> {
        if let Some(cs_id) = self.state.get_transaction_conflict_set(tx_id) {
            let cs = ConflictSet::get(&*self.pool.lock().unwrap(), self.stage, &cs_id)?;
            cs.validate()?;

            Ok(cs)
        } else {
            let err = Error::NotFound;
            Err(err)
        }
    }

    /// `clear_state` clears the state of the `ProtocolState`.
    pub fn clear_state(&mut self) {
        self.state.clear()
    }

    /// `clear` clears the state and stores of the `ProtocolState`.
    pub fn clear(&mut self) -> Result<()> {
        self.clear_state();
        self.pool.lock().unwrap().clear()?;
        self.store.lock().unwrap().clear()?;

        Ok(())
    }

    /// `validate` validates the `ProtocolState`.
    pub fn validate(&self) -> Result<()> {
        self.config.validate()?;
        self.state.validate()?;

        Ok(())
    }
}
