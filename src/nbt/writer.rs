use bytes::{BufMut, BytesMut};

use super::tag::{NbtTag, NbtTagType};
pub fn write_compound_root(protocol: i32, fields: &[(String, NbtTag)]) -> anyhow::Result<BytesMut> {
    let mut out = BytesMut::new();
    out.put_u8(NbtTagType::Compound as u8);
    if protocol < 764 {
        out.put_u16(0);
    }
    write_compound_payload(fields, &mut out)?;
    Ok(out)
}
pub fn write_network_tag(protocol: i32, tag: &NbtTag) -> anyhow::Result<BytesMut> {
    let mut out = BytesMut::new();
    out.put_u8(tag.tag_type() as u8);
    if protocol < 764 {
        out.put_u16(0);
    }
    write_payload(tag, &mut out)?;
    Ok(out)
}

fn write_compound_payload(fields: &[(String, NbtTag)], out: &mut BytesMut) -> anyhow::Result<()> {
    for (name, value) in fields {
        out.put_u8(value.tag_type() as u8);
        write_nbt_string(name, out)?;
        write_payload(value, out)?;
    }
    out.put_u8(NbtTagType::End as u8);
    Ok(())
}

fn write_payload(tag: &NbtTag, out: &mut BytesMut) -> anyhow::Result<()> {
    match tag {
        NbtTag::Byte(value) => out.put_i8(*value),
        NbtTag::Short(value) => out.put_i16(*value),
        NbtTag::Int(value) => out.put_i32(*value),
        NbtTag::Long(value) => out.put_i64(*value),
        NbtTag::Float(value) => out.put_f32(*value),
        NbtTag::Double(value) => out.put_f64(*value),
        NbtTag::ByteArray(values) => {
            write_array_len(values.len(), out)?;
            for value in values {
                out.put_i8(*value);
            }
        }
        NbtTag::String(value) => write_nbt_string(value, out)?,
        NbtTag::List(element_type, values) => {
            out.put_u8(*element_type as u8);
            write_array_len(values.len(), out)?;
            for value in values {
                if value.tag_type() != *element_type {
                    anyhow::bail!("NBT list element type does not match its declared type");
                }
                write_payload(value, out)?;
            }
        }
        NbtTag::Compound(fields) => write_compound_payload(fields, out)?,
        NbtTag::IntArray(values) => {
            write_array_len(values.len(), out)?;
            for value in values {
                out.put_i32(*value);
            }
        }
        NbtTag::LongArray(values) => {
            write_array_len(values.len(), out)?;
            for value in values {
                out.put_i64(*value);
            }
        }
    }
    Ok(())
}

fn write_array_len(length: usize, out: &mut BytesMut) -> anyhow::Result<()> {
    let length = i32::try_from(length)?;
    out.put_i32(length);
    Ok(())
}

fn write_nbt_string(value: &str, out: &mut BytesMut) -> anyhow::Result<()> {
    let bytes = value.as_bytes();
    let length = u16::try_from(bytes.len())?;
    out.put_u16(length);
    out.extend_from_slice(bytes);
    Ok(())
}
