use bytes::{BufMut, BytesMut};

use crate::protocol::io::{ProtocolUtils, VarInt};
pub fn limbo_lock_ground_movement_764() -> BytesMut {
    let mut body = BytesMut::with_capacity(48);
    VarInt::write(&mut body, 0x6D); 
    VarInt::write(&mut body, 1); 
    VarInt::write(&mut body, 1); 
    ProtocolUtils::write_string(&mut body, "minecraft:generic.movement_speed");
    body.put_f64(0.0);
    VarInt::write(&mut body, 0); 
    body
}
