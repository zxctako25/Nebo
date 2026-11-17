use bytes::{BufMut, BytesMut};

use crate::protocol::io::VarInt;

pub fn map_data_packet_754(map_id: i32, colors: &[u8]) -> BytesMut {
    let mut packet = BytesMut::with_capacity(colors.len() + 24);
    VarInt::write(&mut packet, 0x26);
    VarInt::write(&mut packet, map_id);
    packet.put_i8(0); // scale
    packet.put_u8(0); // tracking position
    VarInt::write(&mut packet, 0); // icons
    packet.put_u8(128); // columns
    packet.put_u8(0); // x offset
    packet.put_u8(0); // z offset
    VarInt::write(&mut packet, colors.len() as i32);
    packet.extend_from_slice(colors);
    packet
}

pub fn map_data_packet_764(map_id: i32, colors: &[u8]) -> BytesMut {
    let mut packet = BytesMut::with_capacity(colors.len() + 24);
    VarInt::write(&mut packet, 0x2A); 
    VarInt::write(&mut packet, map_id);
    packet.put_i8(0); 
    packet.put_u8(0); 
    packet.put_u8(0); 
    packet.put_u8(128); 
    packet.put_u8(128); 
    packet.put_u8(0); 
    packet.put_u8(0); 
    VarInt::write(&mut packet, colors.len() as i32);
    packet.extend_from_slice(colors);
    packet
}

pub fn map_data_packet_774(map_id: i32, colors: &[u8]) -> BytesMut {
    let mut packet = BytesMut::with_capacity(colors.len() + 24);
    VarInt::write(&mut packet, 0x31);
    VarInt::write(&mut packet, map_id);
    packet.put_i8(0);
    packet.put_u8(0);
    VarInt::write(&mut packet, 0);
    packet.put_u8(128);
    packet.put_u8(128);
    packet.put_u8(0);
    packet.put_u8(0);
    VarInt::write(&mut packet, colors.len() as i32);
    packet.extend_from_slice(colors);
    packet
}
