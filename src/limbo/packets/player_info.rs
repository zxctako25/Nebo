use bytes::{BufMut, BytesMut};

use crate::protocol::io::{ProtocolUtils, VarInt};
pub fn player_info_add_self_764(uuid: &uuid::Uuid, name: &str) -> BytesMut {
    let mut body = BytesMut::with_capacity(64);
    VarInt::write(&mut body, 0x3C); 
    body.put_u8(0x01 | 0x04 | 0x08); 
    VarInt::write(&mut body, 1); 
    body.extend_from_slice(uuid.as_bytes());
    ProtocolUtils::write_string(&mut body, name);
    VarInt::write(&mut body, 0); 
    VarInt::write(&mut body, 0); 
    body.put_u8(1); 
    body
}
