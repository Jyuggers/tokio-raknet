use bytes::Bytes;
use thiserror::Error;

/// Errors that may occur while decoding RakNet protocol values or packets.
///
/// This type is kept small and generic so it can be shared by all
/// `RaknetEncodable` implementations and packet bodies.
#[derive(Error, Debug)]
pub enum DecodeError {
    /// The buffer did not contain enough bytes to decode the requested value.
    #[error("Unexpected EoF, not enough bytes to read requested type.")]
    UnexpectedEof,

    /// A control packet ID was not recognised by the registry.
    #[error("Unknown Packet, ID: {0}")]
    UnknownId(u8),

    /// A variable-length integer exceeded the supported bit width.
    #[error("VarInt bigger than 128 bits provided.")]
    VarIntExceedsLimit,

    /// Wrapper for packets that are considered legacy/unsupported.
    ///
    /// Callers should typically log the ID and payload and then decide
    /// whether to drop the packet or forward it for custom handling.
    #[error(
        "An unimplemented / legacy packet encountered. \
        Packet ID: {id}"
    )]
    UnimplementedPacket { id: u8, payload: Bytes },

    /// An address encoding used an unsupported version field.
    #[error(
        "An invalid IpAddress version was encountered:\n\
        Provided: {0}, expected: 4 or 6."
    )]
    InvalidAddrVersion(u8),

    /// A disconnect reason value that does not map to any known variant.
    #[error("An unknown disconnection reason was provided. Reason byte: {0}")]
    UnknownDisconnectReason(u8),
    #[error("An unknown reliability value was provided. Reliability byte: {0}")]
    UnknownReliability(u8),
}
