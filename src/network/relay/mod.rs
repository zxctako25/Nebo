mod backend_side;
mod client_side;

use bytes::BytesMut;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::codec::frame::FrameDecoder;
use crate::limbo::ProtocolPackets;
use crate::network::session::ClientSession;
use crate::protocol::constants::*;
pub(crate) struct RelayState {
    pub protocol: ProtocolPackets,
    pub captcha_freeze_ticks: u8,
    pub backend_spawn_teleport_acked: bool,
    pub return_teleport_pending: Option<i32>,
    pub backend_abilities: Option<BytesMut>,
    pub backend_experience: Option<BytesMut>,
    pub client_settings_seen: bool,
    pub client_brand_seen: bool,
    pub client_loaded_seen: bool,
    pub held_client_setup_packets: Vec<BytesMut>,
    pub client_finished_configuration: bool,
    pub backend_finished_configuration: bool,
}

impl RelayState {
    pub fn new(session: &mut ClientSession) -> Self {
        let protocol = ProtocolPackets::new(session.protocol_number);
        Self {
            protocol,
            captcha_freeze_ticks: 0,
            backend_spawn_teleport_acked: false,
            return_teleport_pending: session.limbo_exit_teleport_id.take(),
            backend_abilities: None,
            backend_experience: None,
            client_settings_seen: false,
            client_brand_seen: false,
            client_loaded_seen: protocol.player_loaded_serverbound() < 0,
            held_client_setup_packets: Vec::new(),
            client_finished_configuration: session.state
                == crate::protocol::packet::ConnectionState::Play,
            backend_finished_configuration: session.state
                == crate::protocol::packet::ConnectionState::Play,
        }
    }
}

pub async fn run_relay(
    client: &mut TcpStream,
    backend: &mut TcpStream,
    session: &mut ClientSession,
) -> anyhow::Result<()> {
    let c_decoder = FrameDecoder {
        max_frame_size: MAX_FRAME_SIZE_BEFORE_DECOMPRESSION,
    };
    let b_decoder = FrameDecoder {
        max_frame_size: MAX_FRAME_SIZE_BEFORE_DECOMPRESSION,
    };
    let mut client_buf = BytesMut::with_capacity(8192);
    let mut backend_buf = std::mem::take(&mut session.pending_backend_bytes);
    let mut tick = tokio::time::interval(Duration::from_millis(50));
    let mut state = RelayState::new(session);

    loop {
        tokio::select! {
            read = client.read_buf(&mut client_buf) => {
                let n = read?;
                if n == 0 { break; }
                while let Some(mut frame) = c_decoder.decode(&mut client_buf)? {
                    let payload = if session.compression_enabled {
                        match session.compression_decoder.decode(&mut frame)? {
                            Some(p) => p,
                            None => continue,
                        }
                    } else {
                        frame
                    };
                    if !client_side::handle_client_frame(
                        client,
                        backend,
                        session,
                        &mut state,
                        payload,
                    )
                    .await?
                    {
                        break;
                    }
                }
            }
            read = backend.read_buf(&mut backend_buf) => {
                let n = read?;
                if n == 0 { break; }
                while let Some(mut frame) = b_decoder.decode(&mut backend_buf)? {
                    let payload = if session.backend_compression_enabled {
                        match session.backend_compression_decoder.decode(&mut frame)? {
                            Some(p) => p,
                            None => continue,
                        }
                    } else {
                        frame
                    };
                    if !backend_side::handle_backend_frame(
                        client,
                        backend,
                        session,
                        &mut state,
                        payload,
                    )
                    .await?
                    {
                        break;
                    }
                }
            }
            _ = tick.tick() => {
                if !relay_tick(client, backend, session, &mut state).await? {
                    break;
                }
            }
        }
    }
    Ok(())
}

async fn relay_tick(
    client: &mut TcpStream,
    _backend: &mut TcpStream,
    session: &mut ClientSession,
    state: &mut RelayState,
) -> anyhow::Result<bool> {
    use crate::limbo::LimboPhase;
    use crate::network::client::util::send_client_body;
    use crate::limbo::packets::{experience_packet, position_packet};
    use crate::protocol::io::VarInt;

    if session.limbo.phase == LimboPhase::Falling {
        if session.limbo.expired() || session.limbo.movement_timed_out() {
            let message = if session.limbo.expired() { &session.limbo_timeout_message } else { &session.bot_detected_message };
            use crate::network::client::util::send_client_body;
            use crate::limbo::packets::chat_packet;
            send_client_body(client, session, chat_packet(session.protocol_number, state.protocol.chat_clientbound(), message)?).await?;
            return Ok(false);
        }
        if session.limbo.finish_fall_if_ready() {
            if !session.captcha_enabled {
                if !state.client_settings_seen || !state.client_brand_seen || !state.client_loaded_seen {
                    use crate::network::client::util::send_client_body;
                    use crate::limbo::packets::chat_packet;
                    send_client_body(client, session, chat_packet(session.protocol_number, state.protocol.chat_clientbound(), &session.bot_detected_message)?).await?;
                    return Ok(false);
                }
                session.limbo.phase = crate::limbo::LimboPhase::Passed;
                return Ok(true);
            }
            crate::network::client::limbo_phase::begin_captcha_for_relay(client, session, state.protocol).await?;
            return Ok(true);
        }
        let progress = session.limbo.progress() as f32 / 100.0;
        let level = (session.limbo.progress() as i32).min(100);
        send_client_body(
            client,
            session,
            experience_packet(
                session.protocol_number,
                state.protocol.experience_clientbound(),
                progress,
                level,
            ),
        )
            .await?;
    } else if session.limbo.phase == LimboPhase::Captcha {
        if session.limbo.expired() {
            use crate::network::client::util::send_client_body;
            use crate::limbo::packets::chat_packet;
            send_client_body(client, session, chat_packet(session.protocol_number, state.protocol.chat_clientbound(), &session.limbo_timeout_message)?).await?;
            return Ok(false);
        }
        state.captcha_freeze_ticks = state.captcha_freeze_ticks.wrapping_add(1);
        if state.captcha_freeze_ticks >= 1 {
            state.captcha_freeze_ticks = 0;
            if let Some(mut position) = session.limbo.origin {
                position.y = 65.0;
                position.teleport_id = session.limbo.next_teleport_id();
                session.last_limbo_teleport_id = position.teleport_id;
                session
                    .pending_limbo_teleport_acks
                    .insert(position.teleport_id);
                send_client_body(
                    client,
                    session,
                    position_packet(
                        session.protocol_number,
                        state.protocol.position_clientbound(),
                        position,
                    ),
                )
                    .await?;
            }
        }
    }
    Ok(true)
}
