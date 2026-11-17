use bytes::{BufMut, BytesMut};

use crate::limbo::nbt::{write_network_tag, NbtTag};
use crate::protocol::io::VarInt;

use super::helpers::compound;
pub fn limbo_chunk_packets(protocol: i32) -> anyhow::Result<Vec<BytesMut>> {
    let radius = if protocol == 774 { 2 } else { 1 };
    let side = radius * 2 + 1;
    let mut packets = Vec::with_capacity((side * side) as usize);
    for x in -radius..=radius {
        for z in -radius..=radius {
            packets.push(empty_chunk_packet_at(protocol, x, z)?);
        }
    }
    Ok(packets)
}

pub fn empty_chunk_packet_at(
    protocol: i32,
    chunk_x: i32,
    chunk_z: i32,
) -> anyhow::Result<BytesMut> {
    anyhow::ensure!(
        (754..=774).contains(&protocol),
        "unsupported chunk protocol {protocol}"
    );
    let mut packet = BytesMut::new();
    VarInt::write(&mut packet, chunk_data_packet_id(protocol));
    packet.put_i32(chunk_x);
    packet.put_i32(chunk_z);

    if protocol <= 756 {
        if protocol == 754 {
            packet.put_u8(1); 
            VarInt::write(&mut packet, 0); 
        } else {
            VarInt::write(&mut packet, 0); 
        }
        let heightmap = compound(vec![
            ("MOTION_BLOCKING".to_owned(), NbtTag::LongArray(vec![0; 37])),
            ("WORLD_SURFACE".to_owned(), NbtTag::LongArray(vec![0; 37])),
        ]);
        packet.extend_from_slice(&write_network_tag(protocol, &heightmap)?);
        VarInt::write(&mut packet, 1024);
        for _ in 0..1024 {
            VarInt::write(&mut packet, 1); 
        }
        VarInt::write(&mut packet, 0); 
        VarInt::write(&mut packet, 0); 
        return Ok(packet);
    }

    if protocol >= 770 {
        VarInt::write(&mut packet, 2); 
        for id in [1, 4] {
            VarInt::write(&mut packet, id);
            VarInt::write(&mut packet, 37);
            for _ in 0..37 {
                packet.put_i64(0);
            }
        }
    } else {
        let heightmap = compound(vec![
            ("MOTION_BLOCKING".to_owned(), NbtTag::LongArray(vec![0; 37])),
            ("WORLD_SURFACE".to_owned(), NbtTag::LongArray(vec![0; 37])),
        ]);
        packet.extend_from_slice(&write_network_tag(protocol, &heightmap)?);
    }
    let section_count = if protocol >= 757 { 24 } else { 16 };
    let mut sections = BytesMut::new();
    for _ in 0..section_count {
        sections.put_i16(0); 
        sections.put_u8(0); 
        VarInt::write(&mut sections, 0); 
        if protocol < 770 {
            VarInt::write(&mut sections, 0); 
        }
        sections.put_u8(0); 
        VarInt::write(&mut sections, 1); 
        if protocol < 770 {
            VarInt::write(&mut sections, 0); 
        }
    }
    VarInt::write(&mut packet, sections.len() as i32);
    packet.extend_from_slice(&sections);
    VarInt::write(&mut packet, 0); 
    if protocol < 763 {
        packet.put_u8(1);
    } 
    write_bitset(&mut packet, &[]); 
    write_bitset(&mut packet, &[]); 
    let light_mask = (1_i64 << (section_count + 2)) - 1;
    write_bitset(&mut packet, &[light_mask]); 
    write_bitset(&mut packet, &[light_mask]);
    VarInt::write(&mut packet, 0); 
    VarInt::write(&mut packet, 0); 
    Ok(packet)
}

pub fn update_view_position_packet(protocol: i32, x: i32, z: i32) -> anyhow::Result<BytesMut> {
    let id = match protocol {
        754 => 0x40,
        755..=758 => 0x49,
        759 => 0x48,
        760 => 0x4b,
        761 => 0x4a,
        762..=763 => 0x4e,
        764 => 0x50,
        765 => 0x52,
        766..=767 => 0x54,
        768..=769 => 0x58,
        770..=772 => 0x57,
        773..=774 => 0x5c,
        _ => anyhow::bail!("unsupported view position protocol {protocol}"),
    };
    let mut packet = BytesMut::new();
    VarInt::write(&mut packet, id);
    VarInt::write(&mut packet, x);
    VarInt::write(&mut packet, z);
    Ok(packet)
}

fn write_bitset(out: &mut BytesMut, words: &[i64]) {
    VarInt::write(out, words.len() as i32);
    for word in words {
        out.put_i64(*word);
    }
}

fn chunk_data_packet_id(protocol: i32) -> i32 {
    match protocol {
        754 => 0x20,
        755..=758 => 0x22,
        759 => 0x1f,
        760 => 0x21,
        761 => 0x20,
        762..=763 => 0x24,
        764..=765 => 0x25,
        766..=767 => 0x27,
        768..=769 => 0x28,
        770..=772 => 0x27,
        773..=774 => 0x2c,
        _ => unreachable!(),
    }
}
