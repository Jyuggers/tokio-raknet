use bytes::{Buf, BufMut};

use crate::protocol::packet::{DecodeError, RaknetEncodable};
use crate::protocol::types::Magic;

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
        let size = core::mem::size_of::<Self>();
        if src.remaining() < size {
            return Err(DecodeError::UnexpectedEof);
        }

        let mut magic = [0u8; 16];

        // This reads exactly 16 bytes and advances the Buf properly.
        src.copy_to_slice(&mut magic);

        Ok(magic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn bool_roundtrip() {
        for &v in &[false, true] {
            let mut buf = BytesMut::new();
            v.encode_raknet(&mut buf);
            let mut slice = buf.freeze();
            let decoded = bool::decode_raknet(&mut slice).unwrap();
            assert_eq!(decoded, v);
        }
    }

    #[test]
    fn magic_roundtrip() {
        let value: Magic = [0x12; 16];
        let mut buf = BytesMut::new();
        value.encode_raknet(&mut buf);
        let mut slice = buf.freeze();
        let decoded = Magic::decode_raknet(&mut slice).unwrap();
        assert_eq!(decoded, value);
    }
}


