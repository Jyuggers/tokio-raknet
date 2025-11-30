//! Tokio-based UDP transport layer for RakNet sessions.
//!
//! This module exposes high-level server and client types:
//! - `RaknetListener` / `RaknetConnection` for server-side use.
//! - `RaknetClient` for client-side connections.
//!
//! All low-level RakNet details (fragmentation, reliability, ordering,
//! ACK/NACK handling) are delegated to the `session` module.
//!
//! The transport layer handles the actual UDP sockets and multiplexing
//! multiple sessions over a single port (for the server).

use bytes::Bytes;
use std::net::SocketAddr;

use crate::protocol::{packet::RaknetPacket, reliability::Reliability, state::RakPriority};

pub mod client;
pub mod listener;
mod listener_conn;
pub mod mux;

pub use client::RaknetClient;
pub use listener::RaknetListener;
pub use listener_conn::RaknetConnection;

/// High-level message object for sending data.
/// Wraps the payload and delivery options (reliability, channel, priority).
#[derive(Debug, Clone)]
pub struct Message {
    pub buffer: Bytes,
    pub reliability: Reliability,
    pub channel: u8,
    pub priority: RakPriority,
}

impl Message {
    pub fn new(buffer: impl Into<Bytes>) -> Self {
        Self {
            buffer: buffer.into(),
            reliability: Reliability::ReliableOrdered,
            channel: 0,
            priority: RakPriority::Normal,
        }
    }

    pub fn reliability(mut self, reliability: Reliability) -> Self {
        self.reliability = reliability;
        self
    }

    pub fn channel(mut self, channel: u8) -> Self {
        self.channel = channel;
        self
    }

    pub fn priority(mut self, priority: RakPriority) -> Self {
        self.priority = priority;
        self
    }
}

impl From<Bytes> for Message {
    fn from(buffer: Bytes) -> Self {
        Self::new(buffer)
    }
}

impl From<Vec<u8>> for Message {
    fn from(vec: Vec<u8>) -> Self {
        Self::new(vec)
    }
}

impl From<&'static [u8]> for Message {
    fn from(slice: &'static [u8]) -> Self {
        Self::new(Bytes::from(slice))
    }
}

impl From<&str> for Message {
    fn from(s: &str) -> Self {
        Self::new(Bytes::copy_from_slice(s.as_bytes()))
    }
}

impl From<String> for Message {
    fn from(s: String) -> Self {
        Self::new(Bytes::from(s))
    }
}

/// Message sent from a connection handle to the transport muxer,
/// representing an outbound logical RakNet packet.
pub struct OutboundMsg {
    /// Remote peer this logical packet should be sent to.
    pub peer: SocketAddr,
    /// High-level RakNet packet to send.
    pub packet: RaknetPacket,
    /// Desired reliability semantics for this send.
    pub reliability: Reliability,
    /// Ordering channel, typically 0 unless using multiple streams.
    pub channel: u8,
    /// Priority for the RakNet scheduler; lower index sends sooner.
    pub priority: RakPriority,
}
