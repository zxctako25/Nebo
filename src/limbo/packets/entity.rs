use bytes::{BufMut, BytesMut};

use crate::protocol::io::VarInt;

use crate::limbo::nbt::{write_compound_root, NbtTag};

pub fn framed_captcha_entity_packets_754(
    entity_id: i32,
    map_id: i32,
    block_x: i32,
    block_y: i32,
    block_z: i32,
    facing: i32,
) -> anyhow::Result<[BytesMut; 2]> {
    let (step_x, step_z, yaw) = match facing {
        2 => (0.0, -1.0, -128_i8),
        3 => (0.0, 1.0, 0_i8),
        4 => (-1.0, 0.0, 64_i8),
        5 => (1.0, 0.0, -64_i8),
        _ => anyhow::bail!("invalid item frame facing {facing}"),
    };
    let x = f64::from(block_x) + 0.5 - step_x * 0.46875;
    let y = f64::from(block_y) + 0.5;
    let z = f64::from(block_z) + 0.5 - step_z * 0.46875;

    let mut spawn = BytesMut::with_capacity(64);
    VarInt::write(&mut spawn, 0x00);
    VarInt::write(&mut spawn, entity_id);
    let uuid = uuid::Uuid::from_u128(((entity_id as u128) << 64) | entity_id as u128);
    spawn.extend_from_slice(uuid.as_bytes());
    VarInt::write(&mut spawn, 38); // item_frame registry ID in 1.16.5
    spawn.put_f64(x);
    spawn.put_f64(y);
    spawn.put_f64(z);
    spawn.put_i8(0); // pitch
    spawn.put_i8(yaw); // yaw
    VarInt::write(&mut spawn, facing);
    spawn.put_i16(0);
    spawn.put_i16(0);
    spawn.put_i16(0);

    let mut metadata = BytesMut::with_capacity(32);
    VarInt::write(&mut metadata, 0x44);
    VarInt::write(&mut metadata, entity_id);
    metadata.put_u8(6); // item-frame displayed item metadata index
    VarInt::write(&mut metadata, 6); // Slot serializer
    metadata.put_u8(1); // item present
    VarInt::write(&mut metadata, 733); // filled_map item in 1.16.5
    metadata.put_u8(1);
    metadata.extend_from_slice(&write_compound_root(
        754,
        &[("map".to_owned(), NbtTag::Int(map_id))],
    )?);
    metadata.put_u8(0xFF);
    Ok([spawn, metadata])
}
pub fn framed_captcha_entity_packets_764(
    entity_id: i32,
    map_id: i32,
    block_x: i32,
    block_y: i32,
    block_z: i32,
    facing: i32,
) -> anyhow::Result<[BytesMut; 2]> {
    let (step_x, step_z, yaw) = match facing {
        2 => (0.0, -1.0, -128_i8), 
        3 => (0.0, 1.0, 0_i8),     
        4 => (-1.0, 0.0, 64_i8),   
        5 => (1.0, 0.0, -64_i8),   
        _ => anyhow::bail!("invalid item frame facing {facing}"),
    };
    let x = f64::from(block_x) + 0.5 - step_x * 0.46875;
    let y = f64::from(block_y) + 0.5;
    let z = f64::from(block_z) + 0.5 - step_z * 0.46875;

    let mut spawn = BytesMut::with_capacity(64);
    VarInt::write(&mut spawn, 0x01); 
    VarInt::write(&mut spawn, entity_id);
    let uuid = uuid::Uuid::from_u128(((entity_id as u128) << 64) | entity_id as u128);
    spawn.extend_from_slice(uuid.as_bytes());
    VarInt::write(&mut spawn, 56); 
    spawn.put_f64(x);
    spawn.put_f64(y);
    spawn.put_f64(z);
    spawn.put_i8(0); 
    spawn.put_i8(yaw);
    spawn.put_i8(yaw);
    VarInt::write(&mut spawn, facing); 
    spawn.put_i16(0);
    spawn.put_i16(0);
    spawn.put_i16(0);

    let mut metadata = BytesMut::with_capacity(32);
    VarInt::write(&mut metadata, 0x54); 
    VarInt::write(&mut metadata, entity_id);
    metadata.put_u8(8); 
    VarInt::write(&mut metadata, 7); 
    metadata.put_u8(1); 
    VarInt::write(&mut metadata, 941); 
    metadata.put_u8(1); 
    metadata.extend_from_slice(&write_compound_root(
        764,
        &[("map".to_owned(), NbtTag::Int(map_id))],
    )?);
    metadata.put_u8(0xFF); 
    Ok([spawn, metadata])
}

pub fn framed_captcha_entity_packets_774(
    entity_id: i32,
    map_id: i32,
    block_x: i32,
    block_y: i32,
    block_z: i32,
    facing: i32,
) -> anyhow::Result<[BytesMut; 2]> {
    let (step_x, step_z, yaw) = match facing {
        2 => (0.0, -1.0, -128_i8),
        3 => (0.0, 1.0, 0_i8),
        4 => (-1.0, 0.0, 64_i8),
        5 => (1.0, 0.0, -64_i8),
        _ => anyhow::bail!("invalid item frame facing {facing}"),
    };
    let x = f64::from(block_x) + 0.5 - step_x * 0.46875;
    let y = f64::from(block_y) + 0.5;
    let z = f64::from(block_z) + 0.5 - step_z * 0.46875;

    let mut spawn = BytesMut::with_capacity(64);
    VarInt::write(&mut spawn, 0x01);
    VarInt::write(&mut spawn, entity_id);
    let uuid = uuid::Uuid::from_u128(((entity_id as u128) << 64) | entity_id as u128);
    spawn.extend_from_slice(uuid.as_bytes());
    VarInt::write(&mut spawn, 73);
    spawn.put_f64(x);
    spawn.put_f64(y);
    spawn.put_f64(z);
    spawn.put_u8(0);
    spawn.put_i8(0);
    spawn.put_i8(yaw);
    spawn.put_i8(yaw);
    VarInt::write(&mut spawn, facing);

    let mut metadata = BytesMut::with_capacity(32);
    VarInt::write(&mut metadata, 0x61);
    VarInt::write(&mut metadata, entity_id);
    metadata.put_u8(9);
    VarInt::write(&mut metadata, 7);
    metadata.put_u8(1);
    VarInt::write(&mut metadata, 1104);
    VarInt::write(&mut metadata, 1);
    VarInt::write(&mut metadata, 0);
    VarInt::write(&mut metadata, 44);
    VarInt::write(&mut metadata, map_id);
    metadata.put_u8(0xFF);
    Ok([spawn, metadata])
}
pub fn self_visible_metadata_764(entity_id: i32) -> BytesMut {
    let mut body = BytesMut::with_capacity(16);
    VarInt::write(&mut body, 0x54); 
    VarInt::write(&mut body, entity_id);
    body.put_u8(0); 
    VarInt::write(&mut body, 0); 
    body.put_u8(0); 
    body.put_u8(17); 
    VarInt::write(&mut body, 0); 
    body.put_u8(0x7F); 
    body.put_u8(0xFF); 
    body
}
