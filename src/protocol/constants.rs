use bitflags::bitflags;
use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6},
    time::Duration,
};

use crate::protocol::types::Magic;

pub const RAKNET_PROTOCOL_VERSION: u8 = 11; // Mojang's version.
pub const MINIMUM_MTU_SIZE: u16 = 576;
pub const MAXIMUM_MTU_SIZE: u16 = 1400;
pub const MTU_SIZES: &[u16] = &[MAXIMUM_MTU_SIZE, 1200, MINIMUM_MTU_SIZE];

/// Maximum amount of ordering channels as defined in vanilla RakNet.
pub const MAXIMUM_ORDERING_CHANNELS: u8 = 16;

// TODO: maybe change these sizes. Im assuming usize as these are used internally
// to compare against socket read results.

/// Maximum size of an [EncapsulatedPacket] header.
pub const MAXIMUM_ENCAPSULATED_HEADER_SIZE: usize = 28;

pub const UDP_HEADER_SIZE: usize = 8;

pub const RAKNET_DATAGRAM_HEADER_SIZE: usize = 4;

pub const MAXIMUM_CONNECTION_ATTEMPTS: usize = 10;

/// Time between sending connection attempts. Usually in milliseconds.
pub const TIME_BETWEEN_SEND_CONNECTION_ATTEMPTS: Duration = Duration::from_millis(1000);

/// Time after which a session is closed due to no activity. Usually in milliseconds.
pub const SESSION_TIMEOUT: Duration = Duration::from_millis(10000);

/// Time after which a session is refreshed due to no activity. Usually in milliseconds.
pub const SESSION_STALE: Duration = Duration::from_millis(5000);

/// A number of datagram packets each address can send within one RakNet tick (10ms)
pub const DEFAULT_PACKET_LIMIT: usize = 120;

/// A number of all datagrams that will be handled within one RakNet tick before server starts dropping any incoming data.
pub const DEFAULT_GLOBAL_PACKET_LIMIT: usize = 100000;

bitflags! {
    /// Represents all the flags for a RakNet datagram frame.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct RakNetFlags: u8 {
        const VALID            = 0b1000_0000;
        const ACK              = 0b0100_0000;

        // FLAG_NACK and FLAG_HAS_B_AND_AS are the same bit.
        const NACK             = 0b0010_0000;
        const HAS_B_AND_AS     = 0b0010_0000;

        const PACKET_PAIR      = 0b0001_0000;
        const CONTINUOUS_SEND  = 0b0000_1000;
        const NEEDS_B_AND_AS   = 0b0000_0100;

        const RELIABILITY_FLAGS = Self::ACK.bits() | Self::NACK.bits();
    }
}

/// Magic used to identify RakNet packets
pub const DEFAULT_UNCONNECTED_MAGIC: Magic = [
    0x00, 0xFF, 0xFF, 0x00, 0xFE, 0xFE, 0xFE, 0xFE, 0xFD, 0xFD, 0xFD, 0xFD, 0x12, 0x34, 0x56, 0x78,
];

/// Congestion Control related pub constants
pub const CC_MAXIMUM_THRESHOLD: usize = 2000;
pub const CC_ADDITIONAL_VARIANCE: usize = 30;
pub const CC_SYN: usize = 10;

/*
 * IP constants
 */
pub const IPV4_MESSAGE_SIZE: usize = 7;
pub const IPV6_MESSAGE_SIZE: usize = 29;

pub const LOOPBACK_V4: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
pub const ANY_V4: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);

pub const LOCAL_IP_ADDRESSES_V4: [SocketAddrV4; 10] = [
    LOOPBACK_V4,
    ANY_V4,
    ANY_V4,
    ANY_V4,
    ANY_V4,
    ANY_V4,
    ANY_V4,
    ANY_V4,
    ANY_V4,
    ANY_V4,
];

pub const LOOPBACK_V6: SocketAddrV6 = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 0, 0, 0);
pub const ANY_V6: SocketAddrV6 = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, 0, 0, 0);

pub const LOCAL_IP_ADDRESSES_V6: [SocketAddrV6; 10] = [
    LOOPBACK_V6,
    ANY_V6,
    ANY_V6,
    ANY_V6,
    ANY_V6,
    ANY_V6,
    ANY_V6,
    ANY_V6,
    ANY_V6,
    ANY_V6,
];
