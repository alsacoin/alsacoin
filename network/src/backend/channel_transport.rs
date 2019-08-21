//! # Channel Transport
//!
//! `channel_transport` contains the channel transport backend types and functions. In this case
//! the network of nodes is local and every node occupies a different thread in the same process.

use crate::error::Error;
use crate::message::Message;
use crate::result::Result;
use crate::traits::Transport;
use crypto::hash::{Blake512Hasher, Digest};
use crypto::random::Random;
use std::collections::BTreeMap;
use std::sync::mpsc::{channel, Receiver, Sender};

/// `ChannelTransport` is the type implementing the mpsc Channel transport backend.
pub struct ChannelTransport {
    id: Digest,
    address: Vec<u8>,
    receiver: Receiver<Message>,
    channels: BTreeMap<Digest, Sender<Message>>,
}

impl ChannelTransport {
    /// `CHANNEL_ADDRESS_LEN` is the length of a `Channel` address.
    pub const CHANNEL_ADDRESS_LEN: u32 = 16;

    /// `new` creates a new `Channel` backend.
    pub fn new() -> Result<ChannelTransport> {
        let address = Self::gen_address()?;
        let id = Blake512Hasher::hash(&address);

        let (sender, receiver) = channel();

        let mut channels = BTreeMap::new();
        channels.insert(id, sender.clone());

        let transport = ChannelTransport {
            id,
            address,
            receiver,
            channels,
        };

        Ok(transport)
    }

    /// `gen_address` generates a new `Channel` address.
    pub fn gen_address() -> Result<Vec<u8>> {
        Random::bytes(Self::CHANNEL_ADDRESS_LEN as usize).map_err(|e| e.into())
    }

    /// `calc_id` calculates the `Channel` id.
    pub fn calc_id(&self) -> Digest {
        Blake512Hasher::hash(&self.address)
    }

    /// `lookup_channel` look ups a recorded `Channel`.
    pub fn lookup_channel(&self, id: &Digest) -> bool {
        self.channels.contains_key(id)
    }

    /// `add_channel` records a new `Channel`.
    pub fn add_channel(&mut self, id: Digest, channel: &Sender<Message>) -> Result<()> {
        if self.id == id {
            let err = Error::NotAllowed;
            return Err(err);
        }

        if self.lookup_channel(&id) {
            let err = Error::AlreadyFound;
            return Err(err);
        }

        self.channels.insert(id, channel.to_owned());

        Ok(())
    }

    /// `remove_channel` removes a recorded `Channel`.
    pub fn remove_channel(&mut self, id: &Digest) -> Result<()> {
        if id == &self.id {
            let err = Error::NotAllowed;
            return Err(err);
        }

        if self.channels.remove(id).is_none() {
            let err = Error::NotFound;
            Err(err)
        } else {
            Ok(())
        }
    }

    /// `validate` validates the `Channel`.
    pub fn validate(&self) -> Result<()> {
        if self.id != self.calc_id() {
            let err = Error::InvalidId;
            return Err(err);
        }

        if !self.lookup_channel(&self.id) {
            let err = Error::NotFound;
            return Err(err);
        }

        Ok(())
    }

    fn _send(&self, address: &[u8], data: &[u8]) -> Result<()> {
        if address.len() != Self::CHANNEL_ADDRESS_LEN as usize {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let id = Blake512Hasher::hash(address);

        if let Some(ref sender) = self.channels.get(&id) {
            let msg = Message {
                address: self.address.clone(),
                data: data.to_owned(),
            };

            sender.send(msg).map_err(|e| e.into())
        } else {
            let err = Error::NotFound;
            Err(err)
        }
    }

    fn _recv(&mut self) -> Result<Message> {
        self.receiver.recv().map_err(|e| e.into())
    }
}

impl Transport for ChannelTransport {
    fn local_address(&self) -> Result<Vec<u8>> {
        Ok(self.address.clone())
    }

    fn send(&mut self, address: &[u8], data: &[u8]) -> Result<()> {
        self._send(address, data)
    }

    fn recv(&mut self) -> Result<Message> {
        self._recv()
    }
}

#[test]
fn test_channel_transport_ops() {
    use crypto::random::Random;

    let res = ChannelTransport::new();
    assert!(res.is_ok());

    let mut trsp_a = res.unwrap();

    let res = trsp_a.validate();
    assert!(res.is_ok());

    let trsp_a_id = trsp_a.id;
    let trsp_a_addr = trsp_a.address.clone();
    let trsp_a_channel = trsp_a.channels.get(&trsp_a_id).unwrap().clone();

    let found = trsp_a.lookup_channel(&trsp_a_id);
    assert!(found);

    let res = trsp_a.add_channel(trsp_a_id, &trsp_a_channel);
    assert!(res.is_err());

    let res = trsp_a.remove_channel(&trsp_a_id);
    assert!(res.is_err());

    let data_len = 1000;
    let data = Random::bytes(data_len).unwrap();

    let res = trsp_a.send(&trsp_a_addr, &data);
    assert!(res.is_ok());

    let res = trsp_a.recv();
    assert!(res.is_ok());

    let msg = res.unwrap();
    assert_eq!(msg.address, trsp_a_addr);
    assert_eq!(msg.data, data);

    let mut trsp_b = ChannelTransport::new().unwrap();
    let trsp_b_id = trsp_b.id;
    let trsp_b_addr = trsp_b.address.clone();
    let trsp_b_channel = trsp_b.channels.get(&trsp_b_id).unwrap().clone();

    let found = trsp_b.lookup_channel(&trsp_a_id);
    assert!(!found);

    let found = trsp_a.lookup_channel(&trsp_b_id);
    assert!(!found);

    let res = trsp_b.add_channel(trsp_a_id, &trsp_a_channel);
    assert!(res.is_ok());

    let res = trsp_a.add_channel(trsp_b_id, &trsp_b_channel);
    assert!(res.is_ok());

    let found = trsp_b.lookup_channel(&trsp_a_id);
    assert!(found);

    let found = trsp_a.lookup_channel(&trsp_b_id);
    assert!(found);

    let res = trsp_b.add_channel(trsp_a_id, &trsp_a_channel);
    assert!(res.is_err());

    let res = trsp_a.add_channel(trsp_b_id, &trsp_b_channel);
    assert!(res.is_err());

    let res = trsp_a.send(&trsp_b_addr, &data);
    assert!(res.is_ok());

    let res = trsp_b.recv();
    assert!(res.is_ok());

    let msg = res.unwrap();
    assert_eq!(msg.address, trsp_a_addr);
    assert_eq!(msg.data, data);

    let res = trsp_b.send(&trsp_a_addr, &data);
    assert!(res.is_ok());

    let res = trsp_a.recv();
    assert!(res.is_ok());

    let msg = res.unwrap();
    assert_eq!(msg.address, trsp_b_addr);
    assert_eq!(msg.data, data);

    let res = trsp_b.remove_channel(&trsp_a_id);
    assert!(res.is_ok());

    let res = trsp_a.remove_channel(&trsp_b_id);
    assert!(res.is_ok());

    let found = trsp_b.lookup_channel(&trsp_a_id);
    assert!(!found);

    let found = trsp_a.lookup_channel(&trsp_b_id);
    assert!(!found);

    let res = trsp_b.send(&trsp_a_addr, &data);
    assert!(res.is_err());

    let res = trsp_b.send(&trsp_a_addr, &data);
    assert!(res.is_err());
}
