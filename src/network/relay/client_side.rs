use bytes::{Buf, BytesMut};
use log::warn;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::codec::frame::FrameEncoder;
use crate::limbo::{CaptchaResult, LimboPhase};
use crate::network::client::limbo_phase::is_brand_message;
use crate::network::relay::RelayState;
use crate::network::session::ClientSession;
use crate::protocol::io::VarInt;
use crate::protocol::packet::ConnectionState;

pub(crate) async fn handle_client_frame(
    client: &mut TcpStream,
    backend: &mut TcpStream,
    session: &mut ClientSession,
    state: &mut RelayState,
    payload: BytesMut,
) -> anyhow::Result<bool> {
    let protocol = state.protocol;
    let mut view = payload.clone();
    let id = VarInt::read(&mut view)?;
    if session.state == ConnectionState::Configuration
        && id == protocol.configuration_acknowledged_play_serverbound()
    {
        return Ok(true);
    }
    if id == protocol.teleport_confirm_serverbound() {
        let mut confirm_view = view.clone();
        let confirmed_id = VarInt::read(&mut confirm_view)?;
        if session.pending_limbo_teleport_acks.remove(&confirmed_id) {
            return Ok(true);
        }
    }
    let is_client_finish_configuration = session.state == ConnectionState::Configuration
        && id == protocol.finish_configuration_serverbound();
    if let Some(expected_teleport_id) = state.return_teleport_pending {
        if id == protocol.teleport_confirm_serverbound()
            && VarInt::read(&mut view)? == expected_teleport_id
        {
            state.return_teleport_pending = None;
            return Ok(true);
        }
    }
    if session.state == ConnectionState::Configuration {
        if id == protocol.client_information_configuration() {
            state.client_settings_seen = true;
        }
        if id == protocol.plugin_message_serverbound_configuration()
            && is_brand_message(&mut view)?
        {
            state.client_brand_seen = true;
        }
        if id == protocol.finish_configuration_serverbound() {
            state.client_finished_configuration = true;
        }
    }
    if !state.client_finished_configuration || !state.backend_finished_configuration {
        let encoded = session.backend_compression_encoder.encode(&payload)?;
        backend.write_all(&FrameEncoder::encode(&encoded)).await?;
        return Ok(true);
    }
    session.state = ConnectionState::Play;
    if id == protocol.client_settings_serverbound() {
        state.client_settings_seen = true;
        if matches!(session.limbo.phase, LimboPhase::Falling | LimboPhase::Captcha) {
            state.held_client_setup_packets.push(payload);
            return Ok(true);
        }
    }
    if id == protocol.plugin_message_serverbound() && is_brand_message(&mut view)? {
        state.client_brand_seen = true;
        if matches!(session.limbo.phase, LimboPhase::Falling | LimboPhase::Captcha) {
            state.held_client_setup_packets.push(payload);
            return Ok(true);
        }
    }
    if id == protocol.player_chat_session_serverbound()
        && matches!(session.limbo.phase, LimboPhase::Falling | LimboPhase::Captcha)
    {
        state.held_client_setup_packets.push(payload);
        return Ok(true);
    }
    if id == protocol.player_loaded_serverbound() {
        if matches!(session.limbo.phase, LimboPhase::Falling | LimboPhase::Captcha) {
            state.client_loaded_seen = true;
            state.held_client_setup_packets.push(payload);
            return Ok(true);
        }
    }
    if session.limbo.phase == LimboPhase::Falling {
        if protocol.is_movement_serverbound(id) {
            session.limbo.note_movement_packet();
        }
        if id == protocol.move_rotation_serverbound() && view.len() >= 8 {
            let yaw = view.get_f32();
            let pitch = view.get_f32();
            if let Some(origin) = session.limbo.origin.as_mut() {
                origin.yaw = yaw;
                origin.pitch = pitch;
            }
            return Ok(true);
        }
        if id == protocol.move_ground_serverbound()
            && view.first().is_some_and(|ground| ground & 1 != 0)
        {
            warn!(
                "Limbo fall check rejected grounded client {}",
                session.player_name.as_deref().unwrap_or("unknown")
            );
            send_bot_reason(client, session, protocol).await?;
            return Ok(false);
        }
        if let Some(movement) = protocol.read_movement(id, &mut view)? {
            if !session.fall_teleport_confirmed {
                warn!(
                    "Fall movement arrived before teleport confirmation from {}",
                    session.player_name.as_deref().unwrap_or("unknown")
                );
                send_bot_reason(client, session, protocol).await?;
                return Ok(false);
            }
            if let Some(origin) = session.limbo.origin.as_mut() {
                origin.yaw = movement.position.yaw;
                origin.pitch = movement.position.pitch;
            }
            match session.limbo.observe_movement(movement) {
                crate::limbo::FallResult::Invalid => {
                    warn!(
                        "Limbo fall check rejected {}",
                        session.player_name.as_deref().unwrap_or("unknown")
                    );
                    send_bot_reason(client, session, protocol).await?;
                    return Ok(false);
                }
                crate::limbo::FallResult::Complete => {
                    crate::network::client::limbo_phase::begin_captcha_for_relay(
                        client,
                        session,
                        protocol,
                    )
                        .await?;
                }
                _ => {}
            }
            return Ok(true);
        }
        if protocol.is_movement_serverbound(id) {
            return Ok(true);
        }
    } else if session.limbo.phase == LimboPhase::Captcha && id == protocol.chat_serverbound() {
        let answer = protocol.read_chat(&mut view)?;
        match session.limbo.check_answer(&answer) {
            CaptchaResult::Correct => {
                if !state.client_settings_seen
                    || !state.client_brand_seen
                    || !state.client_loaded_seen
                {
                    warn!(
                        "Limbo client metadata missing for {} (settings={}, brand={}, loaded={})",
                        session.player_name.as_deref().unwrap_or("unknown"),
                        state.client_settings_seen,
                        state.client_brand_seen,
                        state.client_loaded_seen
                    );
                    send_bot_reason(client, session, protocol).await?;
                    return Ok(false);
                }
                for setup_packet in state.held_client_setup_packets.drain(..) {
                    let encoded = session.backend_compression_encoder.encode(&setup_packet)?;
                    backend.write_all(&FrameEncoder::encode(&encoded)).await?;
                }
                let mut pos = session.limbo.origin.unwrap();
                pos.teleport_id = session.limbo.next_teleport_id();
                crate::network::client::util::send_client_body(
                    client,
                    session,
                    crate::limbo::packets::position_packet(
                        session.protocol_number,
                        protocol.position_clientbound(),
                        pos,
                    ),
                )
                    .await?;
                if let Some(abilities) = state.backend_abilities.as_ref() {
                    crate::network::client::util::send_client_body(
                        client,
                        session,
                        abilities.clone(),
                    )
                        .await?;
                }
                if let Some(experience) = state.backend_experience.as_ref() {
                    crate::network::client::util::send_client_body(
                        client,
                        session,
                        experience.clone(),
                    )
                        .await?;
                }
                session.limbo.phase = LimboPhase::Passed;
                state.return_teleport_pending = Some(pos.teleport_id);
                log::info!(
                    "Limbo captcha passed for {}",
                    session.player_name.as_deref().unwrap_or("unknown")
                );
            }
            CaptchaResult::Retry { attempts_left } => {
                let msg = session.captcha_wrong_message.replace("{attempts}", &attempts_left.to_string());
                crate::network::client::util::send_client_body(
                    client,
                    session,
                    crate::limbo::packets::chat_packet(
                        session.protocol_number,
                        protocol.chat_clientbound(),
                        &msg,
                    )?,
                )
                    .await?;
            }
            CaptchaResult::Failed => {
                warn!(
                    "Limbo captcha failed for {}",
                    session.player_name.as_deref().unwrap_or("unknown")
                );
                send_bot_reason(client, session, protocol).await?;
                return Ok(false);
            }
        }
        return Ok(true);
    }
    if matches!(
        session.limbo.phase,
        LimboPhase::Falling | LimboPhase::Captcha
    ) {
        if id == protocol.teleport_confirm_serverbound() {
            let confirmed_id = VarInt::read(&mut view)?;
            let expected_spawn_ack = session.limbo.origin_teleport_id.wrapping_add(1);
            if !state.backend_spawn_teleport_acked && confirmed_id == expected_spawn_ack {
                let mut confirm = BytesMut::new();
                VarInt::write(&mut confirm, id);
                VarInt::write(&mut confirm, session.limbo.origin_teleport_id);
                let encoded = session.backend_compression_encoder.encode(&confirm)?;
                backend.write_all(&FrameEncoder::encode(&encoded)).await?;
                state.backend_spawn_teleport_acked = true;
            }
            return Ok(true);
        }
        if protocol.is_movement_serverbound(id) {
            return Ok(true);
        }
        if id != protocol.keep_alive_serverbound() {
            return Ok(true);
        }
    }
    if session.state == ConnectionState::Play
        && session.limbo.phase == LimboPhase::Passed
        && !is_client_finish_configuration
    {
        if let Some(chat_session) = session.pending_limbo_chat_session.take() {
            if id != protocol.player_chat_session_serverbound() {
                let encoded = session.backend_compression_encoder.encode(&chat_session)?;
                backend.write_all(&FrameEncoder::encode(&encoded)).await?;
            }
        }
    }
    let encoded = session.backend_compression_encoder.encode(&payload)?;
    backend.write_all(&FrameEncoder::encode(&encoded)).await?;
    if is_client_finish_configuration
        && session.state == ConnectionState::Play
        && session.limbo.phase == LimboPhase::Passed
    {
        if let Some(chat_session) = session.pending_limbo_chat_session.take() {
            let encoded = session.backend_compression_encoder.encode(&chat_session)?;
            backend.write_all(&FrameEncoder::encode(&encoded)).await?;
        }
    }
    Ok(true)
}

async fn send_bot_reason(
    client: &mut TcpStream,
    session: &ClientSession,
    protocol: crate::limbo::ProtocolPackets,
) -> anyhow::Result<()> {
    crate::network::client::util::send_client_body(
        client,
        session,
        crate::limbo::packets::chat_packet(
            session.protocol_number,
            protocol.chat_clientbound(),
            &session.bot_detected_message,
        )?,
    )
    .await
}
