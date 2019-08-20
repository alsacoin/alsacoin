//! # Channel
//!
//! `channel` contains the channel backend for the networking types and functions. In this case
//! the network of nodes is local and every node occupies a different thread in the same process.

use crate::error::Error;
use crate::result::Result;
use crate::traits::Transport;
use crypto::hash::{Blake512Hasher, Digest};
use crypto::random::Random;
use models::node::Node;
use std::collections::BTreeMap;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct Channel {
    id: Digest,
    address: Vec<u8>,
    receiver: Receiver<(Node, Vec<u8>)>,
    channels: BTreeMap<Digest, Sender<(Node, Vec<u8>)>>,
}

impl Channel {
    /// `CHANNEL_ADDRESS_LEN` is the length of a `Channel` address.
    pub const CHANNEL_ADDRESS_LEN: u32 = 16;

    /// `new` creates a new `Channel` backend.
    pub fn new() -> Result<Channel> {
        let address = Self::gen_address()?;
        let id = Blake512Hasher::hash(&address);

        let (sender, receiver) = channel();

        let mut channels = BTreeMap::new();
        channels.insert(id, sender.clone());

        let channel = Channel {
            id,
            address,
            receiver,
            channels,
        };

        Ok(channel)
    }

    /// `gen_address` generates a new `Channel` address.
    pub fn gen_address() -> Result<Vec<u8>> {
        Random::bytes(Self::CHANNEL_ADDRESS_LEN as usize).map_err(|e| e.into())
    }

    /// `calc_id` calculates the `Channel` id.
    pub fn calc_id(&self) -> Digest {
        Blake512Hasher::hash(&self.address)
    }

    /// `validate` validates the `Channel`.
    pub fn validate(&self) -> Result<()> {
        if self.id != self.calc_id() {
            let err = Error::InvalidId;
            return Err(err);
        }

        Ok(())
    }
}

impl Transport for Channel {
    fn local_address(&self) -> Result<Vec<u8>> {
        Ok(self.address.clone())
    }

    fn send(&mut self, node: &Node, data: &[u8]) -> Result<()> {
        node.validate()?;

        if let Some(ref sender) = self.channels.get(&node.id) {
            let msg = (node.to_owned(), data.to_owned());
            sender.send(msg).map_err(|e| e.into())
        } else {
            let err = Error::NotFound;
            Err(err)
        }
    }

    fn recv(&mut self) -> Result<(Node, Vec<u8>)> {
        self.receiver.recv().map_err(|e| e.into())
    }
}
