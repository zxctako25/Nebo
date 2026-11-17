use bytes::{BufMut, BytesMut};

use crate::protocol::io::VarInt;

use crate::limbo::protocol::{Position, ProtocolPackets};

pub fn position_packet(protocol: i32, packet_id: i32, position: Position) -> BytesMut {
    let packets = ProtocolPackets::new(protocol);
    let mut body = BytesMut::with_capacity(64);
    VarInt::write(&mut body, packet_id);
    if packets.is_modern_position() {
        VarInt::write(&mut body, position.teleport_id);
        body.put_f64(position.x);
        body.put_f64(position.y);
        body.put_f64(position.z);
        body.put_f64(0.0);
        body.put_f64(0.0);
        body.put_f64(0.0);
        body.put_f32(position.yaw);
        body.put_f32(position.pitch);
        body.put_i32(0);
    } else {
        body.put_f64(position.x);
        body.put_f64(position.y);
        body.put_f64(position.z);
        body.put_f32(position.yaw);
        body.put_f32(position.pitch);
        body.put_u8(0); 
        VarInt::write(&mut body, position.teleport_id);
        if (755..=761).contains(&protocol) {
            body.put_u8(0); 
        }
    }
    body
}
