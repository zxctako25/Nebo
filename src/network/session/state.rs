use bytes::BytesMut;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;
use std::path::PathBuf;

use crate::codec::compression::{CompressionDecoder, CompressionEncoder};
use crate::limbo::{LimboSession, ProtocolPackets};
use crate::protocol::constants::MAX_FRAME_SIZE_AFTER_DECOMPRESSION;
use crate::protocol::packet::ConnectionState;
use crate::protocol::registry::ProtocolRegistry;
use crate::protocol::versions::MinecraftProtocol;

pub struct ClientSession {
    pub protocol: Arc<dyn MinecraftProtocol>,
    pub protocol_number: i32,
    pub state: ConnectionState,
    pub player_name: Option<String>,
    pub player_uuid: Option<Uuid>,
    pub client_address: String,
    pub compression_enabled: bool,
    pub compression_decoder: CompressionDecoder,
    pub compression_encoder: CompressionEncoder,
    pub backend_compression_enabled: bool,
    pub backend_compression_decoder: CompressionDecoder,
    pub backend_compression_encoder: CompressionEncoder,
    pub pending_backend_bytes: BytesMut,
    pub limbo: LimboSession,
    pub local_config_packets_sent: bool,
    pub client_settings_seen: bool,
    pub client_brand_seen: bool,
    pub client_loaded_seen: bool,
    pub fall_teleport_confirmed: bool,
    pub limbo_chunks_acknowledged: bool,
    pub fall_teleport_id: i32,
    pub last_limbo_teleport_id: i32,
    pub last_limbo_teleport_acked: i32,
    pub pending_limbo_teleport_acks: HashSet<i32>,
    pub limbo_exit_teleport_id: Option<i32>,
    pub pre_limbo_setup_packets: Vec<BytesMut>,
    pub pending_limbo_chat_session: Option<BytesMut>,
    pub close_after_response: bool,
    pub captcha_enabled: bool,
    pub captcha_background_count: usize,
    pub captcha_dir: PathBuf,
    pub bot_detected_message: String,
    pub captcha_wrong_message: String,
    pub limbo_action_bar_message: String,
    pub limbo_timeout_message: String,
}

impl ClientSession {
    pub fn new(client_address: String, compression_threshold: i32) -> Self {
        let fallback = ProtocolRegistry::fallback();
        let fallback_protocol = fallback.protocol();
        let client_loaded_seen = ProtocolPackets::new(fallback_protocol).player_loaded_serverbound()
            < 0;
        Self {
            protocol: fallback,
            protocol_number: fallback_protocol,
            state: ConnectionState::Handshake,
            player_name: None,
            player_uuid: None,
            client_address,
            compression_enabled: false,
            compression_decoder: CompressionDecoder::new(
                compression_threshold,
                MAX_FRAME_SIZE_AFTER_DECOMPRESSION,
            ),
            compression_encoder: CompressionEncoder::new(compression_threshold),
            backend_compression_enabled: false,
            backend_compression_decoder: CompressionDecoder::new(
                256,
                MAX_FRAME_SIZE_AFTER_DECOMPRESSION,
            ),
            backend_compression_encoder: CompressionEncoder::new(256),
            pending_backend_bytes: BytesMut::new(),
            limbo: LimboSession::new(),
            local_config_packets_sent: false,
            client_settings_seen: false,
            client_brand_seen: false,
            client_loaded_seen,
            fall_teleport_confirmed: false,
            limbo_chunks_acknowledged: false,
            fall_teleport_id: 0,
            last_limbo_teleport_id: 0,
            last_limbo_teleport_acked: 0,
            pending_limbo_teleport_acks: HashSet::new(),
            limbo_exit_teleport_id: None,
            pre_limbo_setup_packets: Vec::new(),
            pending_limbo_chat_session: None,
            close_after_response: false,
            captcha_enabled: true,
            captcha_background_count: 1,
            captcha_dir: PathBuf::from("Nebo").join("captcha"),
            bot_detected_message: "§cConnection check failed. You were disconnected as a bot.".into(),
            captcha_wrong_message: "§cIncorrect code. Attempts remaining: {attempts}".into(),
            limbo_action_bar_message: "§aYou are in Limbo. Please wait for verification.".into(),
            limbo_timeout_message: "§cVerification timed out. Please reconnect.".into(),
        }
    }
}
