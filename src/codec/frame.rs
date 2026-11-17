use crate::protocol::io::VarInt;
use bytes::{Buf, BytesMut};
use std::io;

pub struct FrameDecoder {
    pub max_frame_size: usize,
}

impl FrameDecoder {
    pub fn decode(&self, buf: &mut BytesMut) -> io::Result<Option<BytesMut>> {
        if !buf.has_remaining() {
            return Ok(None);
        }
        let mut tmp = buf.clone();
        let length = match VarInt::read(&mut tmp) {
            Ok(l) => l,
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(error) => return Err(error),
        };
        if length < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Negative frame length",
            ));
        }
        let length = length as usize;
        if length > self.max_frame_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Frame too large: {} > {}", length, self.max_frame_size),
            ));
        }
        let consumed = buf.len() - tmp.len();
        if buf.remaining() < consumed + length {
            return Ok(None);
        }
        buf.advance(consumed);
        let frame = buf.split_to(length);
        Ok(Some(frame))
    }
}

pub struct FrameEncoder;

impl FrameEncoder {
    pub fn encode(body: &[u8]) -> BytesMut {
        let mut out = BytesMut::with_capacity(5 + body.len());
        VarInt::write(&mut out, body.len() as i32);
        out.extend_from_slice(body);
        out
    }
}
