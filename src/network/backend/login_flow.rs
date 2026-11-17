use bytes::{Buf, BytesMut};
use log::{info, warn};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::codec::compression::{CompressionDecoder, CompressionEncoder};
use crate::codec::frame::{FrameDecoder, FrameEncoder};
use crate::limbo::LimboPhase;
use crate::network::backend::compression_switch::enable_client_compression;
use crate::network::backend::plugin_request::handle_plugin_request;
use crate::network::session::ClientSession;
use crate::network::server::ProxyServer;
use crate::protocol::constants::*;
use crate::protocol::factory::PacketFactory;
use crate::protocol::io::VarInt;
use crate::protocol::packet::*;
use crate::forwarding::velocity;

const DEBUG_HEX: bool = false;

pub async fn connect_backend(
    server: Arc<ProxyServer>,
    session: &mut ClientSession,
    client_stream: &mut TcpStream,
    name: &str,
    uuid: &Uuid,
) -> anyhow::Result<Option<TcpStream>> {
    let post_limbo = session.limbo.phase == LimboPhase::Passed;
    let mut candidates = Vec::new();
    for name in &server.config.try_order {
        if let Some(candidate) = server.config.servers.iter().find(|s| &s.name == name) {
            if !candidates
                .iter()
                .any(|s: &&crate::config::BackendServer| s.name == candidate.name)
            {
                candidates.push(candidate);
            }
        }
    }
    for candidate in &server.config.servers {
        if !candidates.iter().any(|s| s.name == candidate.name) {
            candidates.push(candidate);
        }
    }
    let mut connected = None;
    for candidate in candidates {
        info!(
            "Connecting to backend {}:{} for {}",
            candidate.host, candidate.port, name
        );
        match tokio::time::timeout(
            Duration::from_millis(server.config.connection_timeout_millis),
            TcpStream::connect((candidate.host.as_str(), candidate.port)),
        )
            .await
        {
            Ok(Ok(stream)) => {
                connected = Some((candidate.clone(), stream));
                break;
            }
            Ok(Err(error)) => warn!(
                "Backend {}:{} connection failed: {}",
                candidate.host, candidate.port, error
            ),
            Err(_) => warn!(
                "Backend {}:{} connection timed out",
                candidate.host, candidate.port
            ),
        }
    }
    let (backend_cfg, mut backend) = match connected {
        Some(pair) => pair,
        None => {
            if post_limbo {
                return Ok(None);
            }
            let reason =
                crate::util::chat::ChatComponent::text(&server.config.backend_offline_message);
            let body = PacketFactory::login_disconnect(&*session.protocol, &reason);
            crate::network::backend::write_frame(client_stream, session, body).await?;
            return Ok(None);
        }
    };
    backend.set_nodelay(true).ok();

    let protocol = session.protocol.clone();
    let hs = PacketFactory::handshake(
        &*protocol,
        session.protocol_number,
        &backend_cfg.host,
        backend_cfg.port,
        ConnectionState::Login,
    );
    let framed = FrameEncoder::encode(&hs);
    backend.write_all(&framed).await?;

    let ls = PacketFactory::login_start(&*protocol, name, uuid);
    let framed = FrameEncoder::encode(&ls);
    backend.write_all(&framed).await?;

    let frame_decoder = FrameDecoder {
        max_frame_size: MAX_FRAME_SIZE_BEFORE_DECOMPRESSION,
    };
    let mut backend_compression_decoder =
        CompressionDecoder::new(256, MAX_FRAME_SIZE_AFTER_DECOMPRESSION);
    let mut backend_compression_encoder = CompressionEncoder::new(256);
    let mut backend_compression_enabled = false;

    let mut read_buf = BytesMut::with_capacity(8192);

    loop {
        let n = tokio::time::timeout(
            Duration::from_secs(server.config.read_timeout_seconds),
            backend.read_buf(&mut read_buf),
        )
            .await??;
        if n == 0 {
            return Ok(None);
        }

        while let Some(mut frame) = frame_decoder.decode(&mut read_buf)? {
            if DEBUG_HEX {
                let dump = hex::encode(&frame[..frame.len().min(96)]);
                info!(
                    "[BACKEND-LOGIN] compression={} bytes={} hex={}",
                    backend_compression_enabled,
                    frame.len(),
                    dump
                );
            }

            let payload = if backend_compression_enabled {
                match backend_compression_decoder.decode(&mut frame)? {
                    Some(p) => p,
                    None => continue,
                }
            } else {
                frame
            };

            let mut payload_read = payload.clone();
            let packet_id = VarInt::read(&mut payload_read)?;

            if packet_id == protocol.login_id(LoginPacket::SetCompressionClientbound) {
                let threshold = protocol.read_set_compression(&mut payload_read)?;
                info!("Backend Set Compression threshold={}", threshold);
                backend_compression_decoder.enable(threshold);
                backend_compression_encoder.enable(threshold);
                backend_compression_enabled = true;

                if !post_limbo
                    && !session.compression_enabled
                    && protocol.supports_set_compression()
                {
                    enable_client_compression(
                        session,
                        client_stream,
                        server.config.compression_threshold,
                    )
                        .await?;
                }
                continue;
            }

            if packet_id == protocol.login_id(LoginPacket::PluginRequestClientbound) {
                let channel = {
                    let mut r = payload_read.clone();
                    VarInt::read(&mut r)?;
                    crate::protocol::io::ProtocolUtils::read_string(&mut r)?
                };
                let _ = channel; 
                let mut rr = payload.clone();
                VarInt::read(&mut rr)?;
                handle_plugin_request(
                    server.clone(),
                    session,
                    &*protocol,
                    &mut backend_compression_encoder,
                    &mut backend,
                    payload_read,
                )
                    .await?;
                continue;
            }

            if packet_id == protocol.login_id(LoginPacket::SuccessClientbound) {
                let mut rr = payload.clone();
                VarInt::read(&mut rr)?;
                let success = protocol.read_login_success(&mut rr)?;
                info!("Login Success: {} ({})", success.name, success.uuid);

                if post_limbo && protocol.protocol() >= 759 {
                    let ack = PacketFactory::login_acknowledged(&*protocol);
                    let encoded = backend_compression_encoder.encode(&ack)?;
                    backend.write_all(&FrameEncoder::encode(&encoded)).await?;
                    session.state = if protocol.has_configuration_state() {
                        ConnectionState::Configuration
                    } else {
                        ConnectionState::Play
                    };
                    info!(
                        "Backend login accepted after Limbo; entering {:?} ({})",
                        session.state, name
                    );
                } else if post_limbo {
                    session.state = ConnectionState::Play;
                    info!(
                        "Backend login accepted after Limbo; entering PLAY ({})",
                        name
                    );
                } else {
                    let enc = session.compression_encoder.encode(&payload)?;
                    client_stream.write_all(&FrameEncoder::encode(&enc)).await?;
                    if protocol.has_configuration_state() {
                        session.state = ConnectionState::Configuration;
                        info!("Both sides CONFIGURATION, relay active ({})", name);
                    } else {
                        session.state = ConnectionState::Play;
                        info!("Both sides PLAY, relay active ({})", name);
                    }
                }
                if post_limbo {
                    for setup_packet in session.pre_limbo_setup_packets.drain(..) {
                        let encoded = backend_compression_encoder.encode(&setup_packet)?;
                        backend.write_all(&FrameEncoder::encode(&encoded)).await?;
                    }
                    if !protocol.has_configuration_state() {
                        if let Some(chat_session) = session.pending_limbo_chat_session.take() {
                            let encoded = backend_compression_encoder.encode(&chat_session)?;
                            backend.write_all(&FrameEncoder::encode(&encoded)).await?;
                        }
                    }
                }
                server
                    .active_players
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                session.backend_compression_enabled = backend_compression_enabled;
                session.backend_compression_decoder = backend_compression_decoder;
                session.backend_compression_encoder = backend_compression_encoder;
                session.pending_backend_bytes = read_buf;
                return Ok(Some(backend));
            }

            if packet_id == protocol.login_id(LoginPacket::DisconnectClientbound) {
                let reason = protocol.read_disconnect(&mut payload_read)?;
                info!("Backend disconnect: {}", reason);
                if post_limbo {
                    return Ok(None);
                }
                let enc = session.compression_encoder.encode(&payload)?;
                let framed = FrameEncoder::encode(&enc);
                client_stream.write_all(&framed).await?;
                return Ok(None);
            }

            let enc = session.compression_encoder.encode(&payload)?;
            let framed = FrameEncoder::encode(&enc);
            client_stream.write_all(&framed).await?;
        }
    }
}
