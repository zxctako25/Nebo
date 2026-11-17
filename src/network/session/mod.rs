mod state;

pub use state::ClientSession;

use bytes::BytesMut;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

use crate::codec::compression::{CompressionDecoder, CompressionEncoder};
use crate::limbo::{LimboSession, ProtocolPackets};
use crate::protocol::constants::MAX_FRAME_SIZE_AFTER_DECOMPRESSION;
use crate::protocol::packet::ConnectionState;
use crate::protocol::registry::ProtocolRegistry;
use crate::protocol::versions::MinecraftProtocol;
pub const MAX_PRESERVED_SETUP_PACKET_BYTES: usize = 1024;
pub const MAX_PRESERVED_SETUP_PACKETS: usize = 4;
