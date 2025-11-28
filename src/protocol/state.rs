use bytes::{Buf, BufMut};

use crate::protocol::packet::{DecodeError, RaknetEncodable};

#[repr(u8)]
pub enum ConnState {
    Unconnected,
    Connecting,
    Connect,
    Disconnecting,
    Disconnected,
}

#[repr(u8)]
pub enum OfflineState {
    Handshake1,
    Handshake2,
    HandshakeCompleted,
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum DisconnectReason {
    ClosedByRemotePeer,
    ShuttingDown,
    Disconnected,
    TimedOut,
    ConnectionRequestFailed,
    AlreadyConnected,
    NoFreeIncomingConnections,
    IncompatibleProtocolVersion,
    IPRecentlyConnected,
    BadPacket,
    QueueTooLong,
}

impl RaknetEncodable for DisconnectReason {
    fn encode_raknet(&self, dst: &mut impl BufMut) {
        (*self as u8).encode_raknet(dst);
    }

    fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError> {
        let v = u8::decode_raknet(src)?;
        let e = match v {
            0 => DisconnectReason::ClosedByRemotePeer,
            1 => DisconnectReason::ShuttingDown,
            2 => DisconnectReason::Disconnected,
            3 => DisconnectReason::TimedOut,
            4 => DisconnectReason::ConnectionRequestFailed,
            5 => DisconnectReason::AlreadyConnected,
            6 => DisconnectReason::NoFreeIncomingConnections,
            7 => DisconnectReason::IncompatibleProtocolVersion,
            8 => DisconnectReason::IPRecentlyConnected,
            9 => DisconnectReason::BadPacket,
            10 => DisconnectReason::QueueTooLong,
            _ => return Err(DecodeError::UnknownDisconnectReason(v)),
        };
        Ok(e)
    }
}

#[repr(u8)]
pub enum Event {
    NewIncomingConnection,
}
