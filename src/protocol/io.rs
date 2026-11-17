use bytes::{Buf, BufMut, BytesMut};
use std::io;
use uuid::Uuid;

pub struct VarInt;

impl VarInt {
    pub const SEGMENT_BITS: u8 = 0x7F;
    pub const CONTINUE_BIT: u8 = 0x80;
    pub const MAX_BYTES: usize = 5;

    pub fn read(buf: &mut BytesMut) -> io::Result<i32> {
        let mut value: i32 = 0;
        for index in 0..Self::MAX_BYTES {
            if !buf.has_remaining() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "VarInt: not enough data",
                ));
            }
            let b = buf.get_u8();
            if index == 4 && (b & 0xF0) != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "VarInt exceeds 32 bits",
                ));
            }
            value |= ((b & Self::SEGMENT_BITS) as i32) << (index * 7);
            if (b & Self::CONTINUE_BIT) == 0 {
                return Ok(value);
            }
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "VarInt too long",
        ))
    }

    pub fn write(buf: &mut BytesMut, mut value: i32) {
        loop {
            if (value & !(Self::SEGMENT_BITS as i32)) == 0 {
                buf.put_u8(value as u8);
                return;
            }
            buf.put_u8(((value & Self::SEGMENT_BITS as i32) as u8) | Self::CONTINUE_BIT);
            value = ((value as u32) >> 7) as i32;
        }
    }

    pub fn size(value: i32) -> usize {
        for i in 1..Self::MAX_BYTES {
            if (value & (0xFFFFFFFFu32 as i32).wrapping_shl((i * 7) as u32)) == 0 {
                return i;
            }
        }
        Self::MAX_BYTES
    }
}

pub struct ProtocolUtils;

impl ProtocolUtils {
    pub fn read_string(buf: &mut BytesMut) -> io::Result<String> {
        let length = VarInt::read(buf)?;
        if length < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Negative string length",
            ));
        }
        let length = length as usize;
        if length > 32767 * 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid string length: {}", length),
            ));
        }
        if buf.remaining() < length {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "String longer than buffer",
            ));
        }
        let bytes = buf.split_to(length);
        String::from_utf8(bytes.to_vec()).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn write_string(buf: &mut BytesMut, s: &str) {
        let bytes = s.as_bytes();
        VarInt::write(buf, bytes.len() as i32);
        buf.put_slice(bytes);
    }

    pub fn read_uuid(buf: &mut BytesMut) -> io::Result<Uuid> {
        if buf.remaining() < 16 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "UUID is shorter than 16 bytes",
            ));
        }
        let mut bytes = [0u8; 16];
        buf.copy_to_slice(&mut bytes);
        Ok(Uuid::from_bytes(bytes))
    }

    pub fn write_uuid(buf: &mut BytesMut, uuid: &Uuid) {
        buf.put_slice(uuid.as_bytes());
    }

    pub fn read_byte_array(buf: &mut BytesMut) -> io::Result<Vec<u8>> {
        let length = VarInt::read(buf)?;
        if length < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Negative array length",
            ));
        }
        let length = length as usize;
        if length > 1024 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Array too large",
            ));
        }
        if buf.remaining() < length {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Array longer than buffer",
            ));
        }
        Ok(buf.split_to(length).to_vec())
    }

    pub fn read_rest(buf: &mut BytesMut) -> Vec<u8> {
        buf.to_vec()
    }
}
