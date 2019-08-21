//! # Tcp Node
//!
//! `tcp_transport` contains the Tcp transport backend types and functions.

use crate::error::Error;
use crate::message::Message;
use crate::result::Result;
use crate::traits::Transport;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crypto::hash::{Blake512Hasher, Digest};
//use std::net::{TcpListener, TcpStream, Incoming, Shutdown};
use std::io::Cursor;
use std::net::{Ipv4Addr, SocketAddrV4};

/// `TcpNode` is a network node using a Tcp transport.
pub struct TcpNode {
    id: Digest,
    address: SocketAddrV4,
}

impl TcpNode {
    /// `DEFAULT_PORT` is the default port of the `TcpNode`.
    pub const DEFAULT_PORT: u16 = 2019;

    /// `new` creates a new `TcpNode` from an IPv4 address.
    pub fn new(addr: &str) -> Result<TcpNode> {
        let ip_addr: Ipv4Addr = addr.parse()?;
        let address = SocketAddrV4::new(ip_addr, Self::DEFAULT_PORT);

        let addr_buf = Self::address_to_bytes(&address)?;

        let id = Blake512Hasher::hash(&addr_buf);

        let address = SocketAddrV4::new(ip_addr, Self::DEFAULT_PORT);

        let node = TcpNode { id, address };

        Ok(node)
    }

    /// `from_parts` creates a new `TcpNode` with an ip octet and a port.
    pub fn from_parts(ip: [u8; 4], port: u16) -> Result<TcpNode> {
        let ip_addr = Ipv4Addr::from(ip);
        let address = SocketAddrV4::new(ip_addr, port);

        let addr_buf = Self::address_to_bytes(&address)?;

        let id = Blake512Hasher::hash(&addr_buf);

        let node = TcpNode { id, address };

        Ok(node)
    }

    /// `address_to_bytes` converts a SocketAddrV4 to a vector of bytes.
    pub fn address_to_bytes(address: &SocketAddrV4) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        for n in &address.ip().octets() {
            buf.write_u8(*n)?;
        }

        buf.write_u16::<BigEndian>(address.port())?;

        Ok(buf)
    }

    /// `address_from_bytes` returns an address from a slice of bytes.
    pub fn address_from_bytes(buf: &[u8]) -> Result<SocketAddrV4> {
        let mut reader = Cursor::new(buf);

        let mut ip = [0u8; 4];

        #[allow(clippy::needless_range_loop)]
        for i in 0..4 {
            ip[i] = reader.read_u8()?;
        }

        let port = reader.read_u16::<BigEndian>()?;

        let ip_addr = Ipv4Addr::from(ip);
        let address = SocketAddrV4::new(ip_addr, port);
        Ok(address)
    }

    /// `address_bytes` converts the `TcpNode` address to a vector of bytes.
    pub fn address_bytes(&self) -> Result<Vec<u8>> {
        Self::address_to_bytes(&self.address)
    }

    /// `calc_id` calculates the `TcpNode` id.
    pub fn calc_id(&self) -> Result<Digest> {
        let addr_buf = self.address_bytes()?;
        let id = Blake512Hasher::hash(&addr_buf);
        Ok(id)
    }

    /// `validate` validates the `TcpNode`.
    pub fn validate(&self) -> Result<()> {
        if self.id != self.calc_id()? {
            let err = Error::InvalidId;
            return Err(err);
        }

        Ok(())
    }

    /// `_send` sends binary data to a `TcpNode`.
    fn _send(&self, _address: &[u8], _data: &[u8]) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `_recv` receives a `Message` from a known `ChannelNode`.
    fn _recv(&mut self) -> Result<Message> {
        // TODO
        unreachable!()
    }
}

impl Transport for TcpNode {
    fn local_address(&self) -> Result<Vec<u8>> {
        self.address_bytes()
    }

    fn send(&mut self, address: &[u8], data: &[u8]) -> Result<()> {
        self._send(address, data)
    }

    fn recv(&mut self) -> Result<Message> {
        self._recv()
    }
}
