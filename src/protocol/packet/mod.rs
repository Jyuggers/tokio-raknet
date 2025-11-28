pub mod connected;
pub mod open_connection;
pub mod unconnected;
mod utils;

pub use connected::*;
pub use open_connection::*;
pub use unconnected::*;

use bytes::{Buf, BufMut};
use utils::define_raknet_packets;

pub trait Packet: Sized {
    const ID: u8;
    fn encode_body(&self, dst: &mut impl BufMut);
    fn decode_body(src: &mut impl Buf) -> Result<Self, DecodeError>;
}

pub enum DecodeError {
    UnexpectedEof,
    UnknownId(u8),
    VarIntExceedsLimit,
    /// This exists for packets that are considered legacy.
    /// If this is returned I'd log the packet id and hex dump
    /// then send it over my way and I will see what I can do.
    UnimplementedPacket {
        id: u8,
        payload: bytes::Bytes,
    },
    InvalidAddrVersion(u8),
    UnknownDisconnectReason(u8),
}

pub trait RaknetEncodable: Sized {
    fn encode_raknet(&self, dst: &mut impl BufMut);
    fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError>;
}

define_raknet_packets! {
    ConnectedPing,
    ConnectedPong,
    UnconnectedPing,
    UnconnectedPong,
    OpenConnectionRequest1,
    OpenConnectionReply1,
    OpenConnectionRequest2,
    OpenConnectionReply2,
    ConnectionRequest,
    ConnectionRequestAccepted,
    ConnectionRequestFailed,
    AlreadyConnected,
    NewIncomingConnection,
    NoFreeIncomingConnections,
    DisconnectionNotification,
    ConnectionLost,
    ConnectionBanned,
    IncompatibleProtocolVersion,
    IpRecentlyConnected,
    Timestamp,
    AdvertiseSystem,
}
