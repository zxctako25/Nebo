pub mod configuration;
pub mod handshake;
pub mod limbo_phase;
pub mod login;
pub mod status;
pub mod util;

use bytes::BytesMut;
use log::warn;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::network::session::ClientSession;
use crate::network::server::ProxyServer;
use crate::protocol::constants::*;
use crate::protocol::io::VarInt;
use crate::protocol::packet::*;

const HANDSHAKE_PACKET_TIMEOUT: Duration = Duration::from_secs(5);
const LOGIN_START_TIMEOUT: Duration = Duration::from_secs(10);
const STATUS_PACKET_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn handle_client(
    server: Arc<ProxyServer>,
    mut stream: TcpStream,
    ip: String,
) -> anyhow::Result<()> {
    if !server.try_register(&ip).await {
        warn!("Connection limit reached for {}", ip);
        return Ok(());
    }
    let result = run_client(server.clone(), &mut stream, ip.clone()).await;
    server.unregister(&ip).await;
    result
}

async fn run_client(
    server: Arc<ProxyServer>,
    stream: &mut TcpStream,
    ip: String,
) -> anyhow::Result<()> {
    let mut session = ClientSession::new(ip, server.config.compression_threshold);
    session.captcha_enabled = server.anti_bot.captcha_enabled;
    session.captcha_background_count = server.anti_bot.captcha_background_count;
    session.captcha_dir = server.config.data_dir.join("Nebo").join("captcha");
    session.bot_detected_message = server.limbo_texts.render(&server.limbo_texts.bot_detected);
    session.captcha_wrong_message = server.limbo_texts.render(&server.limbo_texts.wrong_captcha_chat);
    session.limbo_action_bar_message = server.limbo_texts.render(&server.limbo_texts.limbo_action_bar);
    session.limbo_timeout_message = server.limbo_texts.render(&server.limbo_texts.timeout);
    session.limbo.configure(server.anti_bot.captcha_timeout_seconds, server.anti_bot.captcha_attempts);
    let mut read_buf = BytesMut::with_capacity(1024);
    let frame_decoder = crate::codec::frame::FrameDecoder {
        max_frame_size: MAX_FRAME_SIZE_BEFORE_DECOMPRESSION,
    };

    loop {
        let state_timeout = match session.state {
            ConnectionState::Handshake => HANDSHAKE_PACKET_TIMEOUT,
            ConnectionState::Login | ConnectionState::AwaitingLoginAck => LOGIN_START_TIMEOUT,
            ConnectionState::Status => STATUS_PACKET_TIMEOUT,
            _ => Duration::from_secs(server.config.read_timeout_seconds),
        };
        let n = match tokio::time::timeout(state_timeout, stream.read_buf(&mut read_buf)).await {
            Ok(result) => result?,
            Err(_) => {
                warn!(
                    "Client {} timed out waiting for a packet in {:?} state",
                    session.client_address, session.state
                );
                break;
            }
        };
        if n == 0 {
            break;
        }

        while let Some(mut frame) = frame_decoder.decode(&mut read_buf)? {
            let payload = if session.compression_enabled {
                match session.compression_decoder.decode(&mut frame)? {
                    Some(p) => p,
                    None => continue,
                }
            } else {
                frame
            };

            match session.state {
                ConnectionState::Handshake => {
                    if !handshake::handle_handshake(&server, &mut session, stream, payload).await? {
                        return Ok(());
                    }
                }
                ConnectionState::Status => {
                    status::handle_status(&server, &mut session, stream, payload).await?;
                }
                ConnectionState::Login => {
                    let backend_socket =
                        login::handle_login(&server, &mut session, stream, payload).await?;
                    if session.close_after_response {
                        return Ok(());
                    }
                    if let Some(mut backend) = backend_socket {
                        let relay_result =
                            crate::network::relay::run_relay(stream, &mut backend, &mut session)
                                .await;
                        server
                            .active_players
                            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                        return relay_result;
                    }
                    if session.state == ConnectionState::Play {
                        return limbo_phase::run_local_limbo(server.clone(), stream, &mut session)
                            .await;
                    }
                }
                ConnectionState::AwaitingLoginAck => {
                    let protocol = crate::limbo::ProtocolPackets::new(session.protocol_number);
                    let mut login_ack = payload;
                    let id = VarInt::read(&mut login_ack)?;
                    if id != protocol.login_acknowledged_serverbound() {
                        warn!(
                            "Expected Login Acknowledged from {}, got packet 0x{:x}",
                            session.client_address, id
                        );
                        return Ok(());
                    }
                    if session.protocol.has_configuration_state() {
                        session.state = ConnectionState::Configuration;
                        for packet in
                            crate::limbo::world::configuration_packets(session.protocol_number)?
                        {
                            util::send_client_body(stream, &session, packet).await?;
                        }
                        session.local_config_packets_sent = true;
                    } else {
                        session.state = ConnectionState::Play;
                        limbo_phase::begin_local_world(stream, &mut session).await?;
                        return limbo_phase::run_local_limbo(server.clone(), stream, &mut session)
                            .await;
                    }
                }
                ConnectionState::Configuration => {
                    if configuration::handle_local_configuration(stream, &mut session, payload)
                        .await?
                    {
                        return limbo_phase::run_local_limbo(server.clone(), stream, &mut session)
                            .await;
                    }
                }
                ConnectionState::Play => {
                    return limbo_phase::run_local_limbo(server.clone(), stream, &mut session)
                        .await;
                }
            }
        }
    }
    Ok(())
}
