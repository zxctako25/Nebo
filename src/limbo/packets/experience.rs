use bytes::{BufMut, BytesMut};

use crate::protocol::io::VarInt;

pub fn experience_packet(protocol: i32, packet_id: i32, progress: f32, level: i32) -> BytesMut {
    let mut body = BytesMut::with_capacity(16);
    VarInt::write(&mut body, packet_id);
    body.put_f32(progress.clamp(0.0, 1.0));
    VarInt::write(&mut body, level);
    VarInt::write(&mut body, 0);
    let _ = protocol;
    body
}
