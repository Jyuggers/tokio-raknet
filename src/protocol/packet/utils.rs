/// INTERNAL
/// Used to generate the RaknetPacket enum type
/// this will be used in all networking loops
/// to encode and decode packets.
macro_rules! define_raknet_packets {
    (
        $(
            $name:ident,
        )+
    ) => {
        pub enum RaknetPacket {
            $(
                $name($name),
            )+
            UserData { id: u8, payload: bytes::Bytes },
        }

        impl RaknetPacket {
            pub fn decode(src: &mut impl Buf) -> Result<Self, DecodeError> {
                if !src.has_remaining() {
                    return Err(DecodeError::UnexpectedEof);
                }
                let id = src.get_u8();
                Ok(match id {
                    $(
                        <$name as Packet>::ID => {
                            RaknetPacket::$name(<$name as Packet>::decode_body(src)?)
                        }
                    )+
                    other if other >= 0x80 => {
                        let mut tmp = bytes::BytesMut::with_capacity(src.remaining());
                        tmp.put(src);
                        RaknetPacket::UserData { id: other, payload: tmp.freeze() }
                    }
                    other => return Err(DecodeError::UnknownId(other)),
                })
            }

            pub fn id(&self) -> u8 {
                match self {
                    $(
                        RaknetPacket::$name(_inner) => <$name as Packet>::ID,
                    )+
                    RaknetPacket::UserData { id, .. } => *id,
                }
            }

            pub fn encode(&self, dst: &mut impl BufMut) {
                dst.put_u8(self.id());
                match self {
                    $(
                        RaknetPacket::$name(inner) => inner.encode_body(dst),
                    )+
                    RaknetPacket::UserData { payload, .. } => dst.put_slice(payload),
                }
            }
        }
    }
}
pub(crate) use define_raknet_packets;
