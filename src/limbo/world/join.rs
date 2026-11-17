use bytes::{BufMut, BytesMut};

use crate::limbo::nbt::write_network_tag;
use crate::protocol::io::{ProtocolUtils, VarInt};

use super::helpers::compound;
use super::registry::{builtin_registry, dimension_data, DimensionKind};
pub fn join_game_packet(protocol: i32) -> anyhow::Result<BytesMut> {
    anyhow::ensure!(
        (754..=774).contains(&protocol),
        "unsupported Limbo protocol {protocol}"
    );
    let mut packet = BytesMut::with_capacity(512);
    VarInt::write(&mut packet, join_game_packet_id(protocol));
    packet.put_i32(1); 

    if protocol < 764 {
        if protocol >= 754 {
            packet.put_u8(0); 
            packet.put_i8(0); 
        } else {
            packet.put_u8(0); 
        }
        packet.put_i8(0); 
        write_string_array(&mut packet, &["minecraft:overworld"])?;
        let registry = write_network_tag(protocol, &builtin_registry(protocol))?;
        packet.extend_from_slice(&registry);

        if (754..=758).contains(&protocol) {
            packet.extend_from_slice(&write_network_tag(
                protocol,
                &dimension_data(protocol, DimensionKind::Overworld),
            )?);
            ProtocolUtils::write_string(&mut packet, "minecraft:overworld");
        } else {
            ProtocolUtils::write_string(&mut packet, "minecraft:overworld");
            ProtocolUtils::write_string(&mut packet, "minecraft:overworld");
        }
        packet.put_i64(0); 
        if protocol >= 754 {
            VarInt::write(&mut packet, 100);
        } else {
            packet.put_u8(100);
        }
        VarInt::write(&mut packet, 8); 
        if protocol >= 757 {
            VarInt::write(&mut packet, 8); 
        }
        packet.put_u8(0); 
        packet.put_u8(1); 
        packet.put_u8(0); 
        packet.put_u8(1); 
        if protocol >= 759 {
            packet.put_u8(0); 
        }
        if protocol >= 763 {
            VarInt::write(&mut packet, 0); 
        }
    } else {
        packet.put_u8(0); 
        write_string_array(&mut packet, &["minecraft:overworld"])?;
        VarInt::write(&mut packet, 100); 
        let limbo_view_distance = if protocol == 774 { 2 } else { 8 };
        VarInt::write(&mut packet, limbo_view_distance);
        VarInt::write(&mut packet, limbo_view_distance);
        packet.put_u8(0); 
        packet.put_u8(1); 
        packet.put_u8(0); 
        if protocol >= 766 {
            VarInt::write(&mut packet, 0); 
        } else {
            ProtocolUtils::write_string(&mut packet, "minecraft:overworld");
        }
        ProtocolUtils::write_string(&mut packet, "minecraft:overworld"); 
        packet.put_i64(0); 
        packet.put_i8(0); 
        packet.put_i8(-1); 
        packet.put_u8(0); 
        packet.put_u8(1); 
        packet.put_u8(0); 
        VarInt::write(&mut packet, 0); 
        if protocol >= 768 {
            VarInt::write(&mut packet, 63); 
        }
        if protocol >= 766 {
            packet.put_u8(0); 
        }
    }
    Ok(packet)
}

fn join_game_packet_id(protocol: i32) -> i32 {
    match protocol {
        754 => 0x24,
        755..=758 => 0x26,
        759 => 0x23,
        760 => 0x25,
        761 => 0x24,
        762..=763 => 0x28,
        764..=765 => 0x29,
        766..=767 => 0x2B,
        768..=769 => 0x2C,
        770..=772 => 0x2B,
        773..=774 => 0x30,
        _ => unreachable!("protocol checked by join_game_packet"),
    }
}

fn write_string_array(packet: &mut BytesMut, values: &[&str]) -> anyhow::Result<()> {
    VarInt::write(packet, i32::try_from(values.len())?);
    for value in values {
        ProtocolUtils::write_string(packet, value);
    }
    Ok(())
}
