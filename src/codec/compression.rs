use bytes::{Buf, BufMut, BytesMut};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::{self, Read, Write};

use crate::protocol::io::VarInt;

pub struct CompressionDecoder {
    pub enabled: bool,
    pub threshold: i32,
    pub max_uncompressed_size: usize,
}

impl CompressionDecoder {
    pub fn new(initial_threshold: i32, max_uncompressed_size: usize) -> Self {
        Self {
            enabled: false,
            threshold: initial_threshold,
            max_uncompressed_size,
        }
    }

    pub fn enable(&mut self, threshold: i32) {
        self.threshold = threshold;
        self.enabled = true;
    }

    pub fn decode(&self, data: &mut BytesMut) -> io::Result<Option<BytesMut>> {
        if !self.enabled {
            return Ok(Some(data.split()));
        }
        if !data.has_remaining() {
            return Ok(None);
        }
        let data_length = VarInt::read(data)?;

        if data_length == 0 {
            let remaining = data.remaining();
            if remaining > self.max_uncompressed_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Uncompressed packet too large",
                ));
            }
            if remaining == 0 {
                return Ok(None);
            }
            return Ok(Some(data.split()));
        }

        if data_length < self.threshold {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Data length {} below threshold {}",
                    data_length, self.threshold
                ),
            ));
        }
        if data_length as usize > self.max_uncompressed_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Uncompressed size too large",
            ));
        }
        if !data.has_remaining() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Empty compressed block",
            ));
        }

        let compressed = data.split().to_vec();
        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut uncompressed = Vec::with_capacity(data_length as usize);
        decoder.read_to_end(&mut uncompressed)?;
        if uncompressed.len() != data_length as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Inflated {} bytes, expected {}",
                    uncompressed.len(),
                    data_length
                ),
            ));
        }

        Ok(Some(BytesMut::from(&uncompressed[..])))
    }
}

pub struct CompressionEncoder {
    pub enabled: bool,
    pub threshold: i32,
}

impl CompressionEncoder {
    pub fn new(initial_threshold: i32) -> Self {
        Self {
            enabled: false,
            threshold: initial_threshold,
        }
    }

    pub fn enable(&mut self, threshold: i32) {
        self.threshold = threshold;
        self.enabled = true;
    }

    pub fn encode(&self, msg: &[u8]) -> io::Result<BytesMut> {
        if !self.enabled {
            return Ok(BytesMut::from(msg));
        }
        let uncompressed_size = msg.len();
        if (uncompressed_size as i32) < self.threshold {
            let mut out = BytesMut::with_capacity(1 + uncompressed_size);
            VarInt::write(&mut out, 0);
            out.extend_from_slice(msg);
            return Ok(out);
        }

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(msg)?;
        let compressed = encoder.finish()?;

        let mut out = BytesMut::with_capacity(5 + compressed.len());
        VarInt::write(&mut out, uncompressed_size as i32);
        out.put_slice(&compressed);
        Ok(out)
    }
}
