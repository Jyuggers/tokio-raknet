pub mod connected;
pub mod open_connection;
pub mod unconnected;
mod error;
mod registry;

pub use connected::*;
pub use open_connection::*;
pub use unconnected::*;
pub use error::DecodeError;
pub use registry::RaknetPacket;

use bytes::{Buf, BufMut};

/// Trait implemented by all concrete RakNet packet body types.
///
/// Implementations are responsible for encoding/decoding only the
/// packet body – the leading ID byte is handled by `RaknetPacket`.
pub trait Packet: Sized {
    /// The fixed ID byte used to identify this packet on the wire.
    const ID: u8;

    /// Encode the body of this packet into the destination buffer.
    fn encode_body(&self, dst: &mut impl BufMut);

    /// Decode the body of this packet from the source buffer.
    fn decode_body(src: &mut impl Buf) -> Result<Self, DecodeError>;
}

/// Trait for types that know how to encode/decode themselves using
/// the RakNet wire format.
pub trait RaknetEncodable: Sized {
    /// Encode this value into the destination buffer.
    fn encode_raknet(&self, dst: &mut impl BufMut);

    /// Decode a value of this type from the source buffer.
    fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError>;
}
