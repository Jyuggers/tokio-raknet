use crate::protocol::packet::{DecodeError, RaknetEncodable};
use bytes::{Buf, BufMut};
use std::{
    mem,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    time::Duration,
};

pub type Magic = [u8; 16];

macro_rules! impl_raknet_int {
    ($ty:ty, $put:ident, $get:ident) => {
        impl RaknetEncodable for $ty {
            fn encode_raknet(&self, dst: &mut impl BufMut) {
                dst.$put(*self as _);
            }

            fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError> {
                let size = mem::size_of::<$ty>();
                if src.remaining() < size {
                    return Err(DecodeError::UnexpectedEof);
                }
                Ok(src.$get() as $ty)
            }
        }
    };
}

// Unsigned big-endian ints:
impl_raknet_int!(u16, put_u16, get_u16);
impl_raknet_int!(u32, put_u32, get_u32);
impl_raknet_int!(u64, put_u64, get_u64);

// Signed big-endian ints (cast through the unsigned read/write):
impl_raknet_int!(i16, put_i16, get_i16);
impl_raknet_int!(i32, put_i32, get_i32);
impl_raknet_int!(i64, put_i64, get_i64);

pub struct U16LE(pub u16);

impl RaknetEncodable for U16LE {
    fn encode_raknet(&self, dst: &mut impl BufMut) {
        dst.put_u16_le(self.0);
    }

    fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError> {
        if src.remaining() < 2 {
            return Err(DecodeError::UnexpectedEof);
        }
        Ok(U16LE(src.get_u16_le()))
    }
}

pub struct U24LE(pub u32);

impl RaknetEncodable for U24LE {
    fn encode_raknet(&self, dst: &mut impl BufMut) {
        let v = self.0;
        // 3-byte little-endian
        dst.put_u8((v & 0xFF) as u8);
        dst.put_u8(((v >> 8) & 0xFF) as u8);
        dst.put_u8(((v >> 16) & 0xFF) as u8);
    }

    fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError> {
        if src.remaining() < 3 {
            return Err(DecodeError::UnexpectedEof);
        }
        let b0 = src.get_u8() as u32;
        let b1 = src.get_u8() as u32;
        let b2 = src.get_u8() as u32;
        Ok(U24LE(b0 | (b1 << 8) | (b2 << 16)))
    }
}

pub struct VarUInt(pub u64);

pub struct VarInt(pub i64);

impl RaknetEncodable for VarUInt {
    fn encode_raknet(&self, dst: &mut impl BufMut) {
        // clone it to mut it.
        let mut v = self.0;
        while v >= 0x80 {
            dst.put_u8(((v & 0x7f) | 0x80) as u8);
            v >>= 7
        }
        dst.put_u8((v & 0x7f) as u8);
    }

    fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError> {
        let mut result = 0;
        let mut shift = 0;
        loop {
            if shift >= 64 {
                return Err(DecodeError::VarIntExceedsLimit);
            }
            if !src.has_remaining() {
                return Err(DecodeError::UnexpectedEof);
            }
            let v = src.get_u8();
            result |= ((v & 0x7f) as u64) << shift;
            if v & 0x80 == 0 {
                break;
            }
            shift += 7
        }
        Ok(VarUInt(result))
    }
}

impl RaknetEncodable for VarInt {
    fn encode_raknet(&self, dst: &mut impl BufMut) {
        let ux = ((self.0 << 1) ^ (self.0 >> 63)) as u64;
        VarUInt(ux).encode_raknet(dst);
    }

    fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError> {
        let ux = VarUInt::decode_raknet(src)?.0;
        let x = ((ux >> 1) as i64) ^ (-((ux & 1) as i64));
        Ok(VarInt(x))
    }
}

impl RaknetEncodable for u8 {
    fn encode_raknet(&self, dst: &mut impl BufMut) {
        dst.put_u8(*self);
    }
    fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError> {
        if !src.has_remaining() {
            return Err(DecodeError::UnexpectedEof);
        }
        Ok(src.get_u8())
    }
}

impl RaknetEncodable for i8 {
    fn encode_raknet(&self, dst: &mut impl BufMut) {
        dst.put_i8(*self);
    }
    fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError> {
        if !src.has_remaining() {
            return Err(DecodeError::UnexpectedEof);
        }
        Ok(src.get_i8())
    }
}

impl RaknetEncodable for bool {
    fn encode_raknet(&self, dst: &mut impl BufMut) {
        dst.put_u8(if *self { 1 } else { 0 });
    }

    fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError> {
        if !src.has_remaining() {
            return Err(DecodeError::UnexpectedEof);
        }
        Ok(src.get_u8() == 1)
    }
}

impl RaknetEncodable for Magic {
    fn encode_raknet(&self, dst: &mut impl BufMut) {
        dst.put_slice(self);
    }

    fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError> {
        let size = mem::size_of::<Self>();
        if src.remaining() < size {
            return Err(DecodeError::UnexpectedEof);
        }

        let mut magic = [0u8; 16];

        // This reads exactly 16 bytes and advances the Buf properly.
        src.copy_to_slice(&mut magic);

        Ok(magic)
    }
}

pub struct Advertisement(pub Option<bytes::Bytes>);

impl RaknetEncodable for Advertisement {
    fn encode_raknet(&self, dst: &mut impl BufMut) {
        if let Some(ad_bytes) = &self.0
            && !ad_bytes.is_empty()
        {
            // Ensure length fits in u16
            let len = ad_bytes.len().min(u16::MAX as usize) as u16;
            dst.put_u16(len);
            dst.put_slice(&ad_bytes[..len as usize]);
        }
        // If self.0 is None or empty, NOP
    }

    fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError> {
        let ad = if src.has_remaining() {
            // Check for at least the length prefix
            if src.remaining() < 2 {
                return Err(DecodeError::UnexpectedEof);
            }
            let len = src.get_u16() as usize;

            // Check if we have enough data for the payload
            if src.remaining() < len {
                return Err(DecodeError::UnexpectedEof);
            }
            Some(src.copy_to_bytes(len))
        } else {
            // No data left, so the field was omitted
            None
        };
        Ok(Advertisement(ad))
    }
}

pub struct RaknetTime(pub u64); // ms on wire

impl RaknetEncodable for RaknetTime {
    fn encode_raknet(&self, dst: &mut impl BufMut) {
        self.0.encode_raknet(dst);
    }

    fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError> {
        Ok(Self(u64::decode_raknet(src)?))
    }
}

impl From<RaknetTime> for Duration {
    fn from(value: RaknetTime) -> Self {
        Duration::from_millis(value.0)
    }
}

/// End of Buffer Padding, adds any length padding till the
/// end of it. So doesn't send any prepadding length or etc.
pub struct EoBPadding(pub usize);

impl RaknetEncodable for EoBPadding {
    fn encode_raknet(&self, dst: &mut impl BufMut) {
        dst.put_bytes(0, self.0);
    }

    fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError> {
        let len = src.remaining();
        src.advance(len);
        Ok(EoBPadding(len))
    }
}

impl RaknetEncodable for SocketAddr {
    fn encode_raknet(&self, dst: &mut impl BufMut) {
        match self {
            SocketAddr::V4(addr) => {
                dst.put_u8(4); // Version 4

                // Get the raw IP bytes
                let ip_bytes = addr.ip().octets();

                // This is the port of `flipBytes`: XOR with 0xFF
                // We use the `!` (bitwise NOT) operator
                let flipped_ip: [u8; 4] = [!ip_bytes[0], !ip_bytes[1], !ip_bytes[2], !ip_bytes[3]];

                dst.put_slice(&flipped_ip);
                dst.put_u16(addr.port());
            }
            SocketAddr::V6(addr) => {
                dst.put_u8(6); // Version 6

                // This manually serializes the C-style `sockaddr_in6` struct
                // Cloudburst uses 23, so we will too.
                dst.put_u16_le(23); // sin6_family (AF_INET6)
                dst.put_u16(addr.port()); // sin6_port
                dst.put_u32(addr.flowinfo()); // sin6_flowinfo
                dst.put_slice(&addr.ip().octets()); // sin6_addr (16 bytes)
                dst.put_u32(addr.scope_id()); // sin6_scope_id
            }
        }
    }

    fn decode_raknet(src: &mut impl Buf) -> Result<Self, DecodeError> {
        if src.remaining() < 1 {
            return Err(DecodeError::UnexpectedEof);
        }
        let version = src.get_u8();

        match version {
            4 => {
                // IPv4
                if src.remaining() < 4 + 2 {
                    // 4 IP bytes + 2 port bytes
                    return Err(DecodeError::UnexpectedEof);
                }
                let mut ip_bytes = [0u8; 4];
                src.copy_to_slice(&mut ip_bytes);

                // Un-flip the bytes
                let unflipped_ip: [u8; 4] =
                    [!ip_bytes[0], !ip_bytes[1], !ip_bytes[2], !ip_bytes[3]];

                let port = src.get_u16();
                Ok(SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::from(unflipped_ip),
                    port,
                )))
            }
            6 => {
                // IPv6
                // Check for all fields: family(2) + port(2) + flow(4) + ip(16) + scope(4)
                if src.remaining() < 2 + 2 + 4 + 16 + 4 {
                    return Err(DecodeError::UnexpectedEof);
                }

                let _family = src.get_u16_le(); // Read and discard
                let port = src.get_u16();
                let flowinfo = src.get_u32();
                let mut ip_bytes = [0u8; 16];
                src.copy_to_slice(&mut ip_bytes);
                let scope_id = src.get_u32();

                Ok(SocketAddr::V6(SocketAddrV6::new(
                    Ipv6Addr::from(ip_bytes),
                    port,
                    flowinfo,
                    scope_id,
                )))
            }
            _ => {
                // You'll want to add this error variant
                Err(DecodeError::InvalidAddrVersion(version))
            }
        }
    }
}
