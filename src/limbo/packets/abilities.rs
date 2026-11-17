use bytes::{BufMut, BytesMut};

use crate::protocol::io::VarInt;

pub fn ability_packet(packet_id: i32, flags: u8, fly_speed: f32, walk_speed: f32) -> BytesMut {
    let mut body = BytesMut::with_capacity(10);
    VarInt::write(&mut body, packet_id);
    body.put_u8(flags);
    body.put_f32(fly_speed);
    body.put_f32(walk_speed);
    body
}
