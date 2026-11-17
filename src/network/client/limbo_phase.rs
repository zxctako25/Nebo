use bytes::{Buf, BufMut, BytesMut};
use log::{info, warn};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::codec::frame::FrameDecoder;
use crate::limbo::captcha;
use crate::limbo::packets::{
    ability_packet, action_bar_packet, chat_packet, experience_packet,
    framed_captcha_entity_packets_754, framed_captcha_entity_packets_764, framed_captcha_entity_packets_774,
    limbo_lock_ground_movement_764, map_data_packet_754, map_data_packet_764, map_data_packet_774,
    player_info_add_self_764, position_packet, self_visible_metadata_764,
};
use crate::limbo::world::{join_game_packet, limbo_chunk_packets, update_view_position_packet};
use crate::limbo::{CaptchaResult, LimboPhase, Position, ProtocolPackets};
use crate::network::backend::connect_backend;
use crate::network::relay::run_relay;
use crate::network::session::{
    ClientSession, MAX_PRESERVED_SETUP_PACKET_BYTES, MAX_PRESERVED_SETUP_PACKETS,
};
use crate::network::server::ProxyServer;
use crate::protocol::constants::MAX_FRAME_SIZE_BEFORE_DECOMPRESSION;
use crate::protocol::io::{ProtocolUtils, VarInt};
use crate::protocol::packet::ConnectionState;

use super::util::send_client_body;

pub async fn begin_local_world(
    client: &mut TcpStream,
    session: &mut ClientSession,
) -> anyhow::Result<()> {
    send_client_body(client, session, join_game_packet(session.protocol_number)?).await?;
    if session.protocol_number == 764 {
        send_self_visible(client, session).await?;
        send_client_body(client, session, limbo_lock_ground_movement_764()).await?;
    }

    let base_teleport_id = 0x4000_0000 | (Uuid::new_v4().as_u128() as u32 & 0x0fff_ffff);
    let origin = Position {
        x: 0.0,
        y: 64.0,
        z: 0.0,
        yaw: 0.0,
        pitch: 0.0,
        teleport_id: base_teleport_id as i32,
    };
    session.fall_teleport_id = origin.teleport_id;
    session.limbo_chunks_acknowledged = false;
    session.last_limbo_teleport_id = origin.teleport_id;
    session
        .pending_limbo_teleport_acks
        .insert(origin.teleport_id);
    let limbo_height = if session.protocol_number == 754 { 220.0 } else { 400.0 };
    session.limbo.start_fall(origin, limbo_height);
    send_client_body(
        client,
        session,
        update_view_position_packet(session.protocol_number, 0, 0)?,
    )
        .await?;
    if session.protocol_number == 764 {
        let mut spawn = BytesMut::new();
        VarInt::write(&mut spawn, 0x52); 
        let packed = 64_i64 & 0xfff;
        spawn.put_i64(packed);
        spawn.put_f32(0.0);
        send_client_body(client, session, spawn).await?;
        let mut batch_start = BytesMut::new();
        VarInt::write(&mut batch_start, 0x0D); 
        send_client_body(client, session, batch_start).await?;
    } else if session.protocol_number == 774 {
        let mut batch_start = BytesMut::new();
        VarInt::write(&mut batch_start, 0x0C);
        send_client_body(client, session, batch_start).await?;
    }
    let chunks = limbo_chunk_packets(session.protocol_number)?;
    let chunk_batch_size = i32::try_from(chunks.len())?;
    for chunk in chunks {
        send_client_body(client, session, chunk).await?;
    }
    if session.protocol_number == 764 {
        let mut batch_finish = BytesMut::new();
        VarInt::write(&mut batch_finish, 0x0C); 
        VarInt::write(&mut batch_finish, chunk_batch_size);
        send_client_body(client, session, batch_finish).await?;
    } else if session.protocol_number == 774 {
        let mut batch_finish = BytesMut::new();
        VarInt::write(&mut batch_finish, 0x0B);
        VarInt::write(&mut batch_finish, chunk_batch_size);
        send_client_body(client, session, batch_finish).await?;
    }
    let mut fall_position = origin;
    fall_position.y = limbo_height;
    send_client_body(
        client,
        session,
        position_packet(
            session.protocol_number,
            ProtocolPackets::new(session.protocol_number).position_clientbound(),
            fall_position,
        ),
    )
        .await?;
    send_client_body(
        client,
        session,
        action_bar_packet(
            session.protocol_number,
            ProtocolPackets::new(session.protocol_number).chat_clientbound(),
            &session.limbo_action_bar_message,
        )?,
    )
        .await?;
    info!(
        "Local Limbo started for {} (protocol {})",
        session.player_name.as_deref().unwrap_or("unknown"),
        session.protocol_number
    );
    Ok(())
}

pub fn is_brand_message(payload: &mut BytesMut) -> anyhow::Result<bool> {
    let channel = ProtocolUtils::read_string(payload)?;
    Ok(matches!(channel.as_str(), "minecraft:brand" | "MC|Brand"))
}

pub fn preserve_setup_packet(session: &mut ClientSession, packet: BytesMut) -> anyhow::Result<()> {
    anyhow::ensure!(
        packet.len() <= MAX_PRESERVED_SETUP_PACKET_BYTES,
        "client setup packet exceeds the {} byte pre-Limbo limit",
        MAX_PRESERVED_SETUP_PACKET_BYTES
    );
    anyhow::ensure!(
        session.pre_limbo_setup_packets.len() < MAX_PRESERVED_SETUP_PACKETS,
        "too many client setup packets before Limbo completion"
    );
    session.pre_limbo_setup_packets.push(packet);
    Ok(())
}

pub async fn run_local_limbo(
    server: Arc<ProxyServer>,
    client: &mut TcpStream,
    session: &mut ClientSession,
) -> anyhow::Result<()> {
    server
        .active_limbo_players
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let _online_guard = LimboOnlineGuard(server.clone());
    let protocol = ProtocolPackets::new(session.protocol_number);
    let decoder = FrameDecoder {
        max_frame_size: MAX_FRAME_SIZE_BEFORE_DECOMPRESSION,
    };
    let mut read_buf = BytesMut::with_capacity(4096);
    let mut tick = tokio::time::interval(Duration::from_millis(50));
    loop {
        tokio::select! {
            read = client.read_buf(&mut read_buf) => {
                let n = read?;
                if n == 0 { break; }
                while let Some(mut frame) = decoder.decode(&mut read_buf)? {
                    let payload = if session.compression_enabled {
                        match session.compression_decoder.decode(&mut frame)? {
                            Some(payload) => payload,
                            None => continue,
                        }
                    } else {
                        frame
                    };
                    let mut view = payload.clone();
                    let id = VarInt::read(&mut view)?;

                    if id == protocol.client_settings_serverbound() {
                        if !session.client_settings_seen && session.protocol_number < 764 {
                            preserve_setup_packet(session, payload.clone())?;
                        }
                        session.client_settings_seen = true;
                    }
                    if id == protocol.plugin_message_serverbound()
                        && is_brand_message(&mut view)? {
                        if !session.client_brand_seen && session.protocol_number < 764 {
                            preserve_setup_packet(session, payload.clone())?;
                        }
                        session.client_brand_seen = true;
                    }
                    if id == protocol.player_loaded_serverbound() {
                        session.client_loaded_seen = true;
                        info!("{} acknowledged Limbo world loading", session.player_name.as_deref().unwrap_or("unknown"));
                    }
                    if session.protocol_number == 774 && id == 0x0A {
                        info!("{} acknowledged the initial 1.21.11 chunk batch", session.player_name.as_deref().unwrap_or("unknown"));
                    }
                    if id == protocol.player_chat_session_serverbound() {
                        session.pending_limbo_chat_session = Some(payload.clone());
                    }
                    if id == protocol.teleport_confirm_serverbound() {
                        let teleport_id = VarInt::read(&mut view)?;
                        if teleport_id == session.fall_teleport_id
                            && !session.fall_teleport_confirmed {
                            session.fall_teleport_confirmed = true;
                            if session.protocol_number != 764 || session.limbo_chunks_acknowledged {
                                session.limbo.start_check_after_client_ready();
                                info!("{} loaded Limbo; starting three-second fall", session.player_name.as_deref().unwrap_or("unknown"));
                                send_limbo_title(client, session, &server.limbo_texts.render(&server.limbo_texts.checking_title), &server.limbo_texts.render(&server.limbo_texts.checking_subtitle)).await?;
                                send_client_body(
                                    client,
                                    session,
                                    chat_packet(
                                        session.protocol_number,
                                        protocol.chat_clientbound(),
                                        &server.limbo_texts.render(&server.limbo_texts.checking_chat),
                                    )?,
                                ).await?;
                            }
                        }
                        session.pending_limbo_teleport_acks.remove(&teleport_id);
                        session.last_limbo_teleport_acked = teleport_id;
                    }
                    if session.limbo.phase == LimboPhase::Captcha
                        && (id == protocol.chat_serverbound()
                            || (session.protocol_number == 764 && id == 0x04))
                    {
                        let answer = protocol.read_chat(&mut view)?;
                        match session.limbo.check_answer(&answer) {
                            CaptchaResult::Correct => {
                                if !session.client_settings_seen || !session.client_brand_seen || !session.client_loaded_seen {
                                    let reason = server.limbo_texts.render(&server.limbo_texts.bot_detected);
                                    send_client_body(client, session, chat_packet(session.protocol_number, protocol.chat_clientbound(), &reason)?).await?;
                                    return Ok(());
                                }
                                info!("Limbo CAPTCHA answer accepted for {}", session.player_name.as_deref().unwrap_or("unknown"));
                                send_client_body(
                                    client,
                                    session,
                                    chat_packet(
                                        session.protocol_number,
                                        protocol.chat_clientbound(),
                                        &server.limbo_texts.render(&server.limbo_texts.successful),
                                    )?,
                                )
                                .await?;
                                if session.protocol.has_configuration_state() {
                                    let mut switch = BytesMut::new();
                                    VarInt::write(
                                        &mut switch,
                                        protocol.start_configuration_clientbound(),
                                    );
                                    send_client_body(client, session, switch).await?;
                                    session.state = ConnectionState::Configuration;
                                }
                                let name = session.player_name.clone().unwrap_or_default();
                                let uuid = session.player_uuid.ok_or_else(|| {
                                    anyhow::anyhow!("Limbo player UUID is missing")
                                })?;
                                info!("{} solved Limbo CAPTCHA; connecting to backend", name);
                                if let Some(mut backend) = connect_backend(
                                    server.clone(), session, client, &name, &uuid,
                                )
                                .await?
                                {
                                    let relay = run_relay(client, &mut backend, session).await;
                                    server.active_players.fetch_sub(
                                        1,
                                        std::sync::atomic::Ordering::Relaxed,
                                    );
                                    return relay;
                                }
                                return Ok(());
                            }
                            CaptchaResult::Retry { attempts_left } => {
                                info!("Limbo CAPTCHA answer rejected for {}; {} attempts remain", session.player_name.as_deref().unwrap_or("unknown"), attempts_left);
                                session.limbo.regenerate_answer();
                                send_captcha_maps(client, session).await?;
                                let message = server.limbo_texts.render(&server.limbo_texts.wrong_captcha_chat).replace("{attempts}", &attempts_left.to_string());
                                send_client_body(client, session, chat_packet(session.protocol_number, protocol.chat_clientbound(), &message)?).await?;
                            }
                            CaptchaResult::Failed => {
                                warn!("Limbo CAPTCHA failed for {}", session.player_name.as_deref().unwrap_or("unknown"));
                                let reason = server.limbo_texts.render(&server.limbo_texts.bot_detected);
                                send_client_body(client, session, chat_packet(session.protocol_number, protocol.chat_clientbound(), &reason)?).await?;
                                return Ok(());
                            }
                        }
                        continue;
                    }

                    if session.limbo.phase == LimboPhase::Captcha {
                        if let Some(movement) = protocol.read_movement(id, &mut view)? {
                            if id == protocol.move_full_serverbound() {
                                if let Some(origin) = session.limbo.origin.as_mut() {
                                    origin.yaw = movement.position.yaw;
                                    origin.pitch = movement.position.pitch;
                                }
                            }
                            continue;
                        }
                        if id == protocol.move_rotation_serverbound() && view.len() >= 9 {
                            let yaw = view.get_f32();
                            let pitch = view.get_f32();
                            if let Some(origin) = session.limbo.origin.as_mut() {
                                origin.yaw = yaw;
                                origin.pitch = pitch;
                            }
                            continue;
                        }
                    }

                    if session.protocol_number == 764 && id == 0x07 {
                        session.limbo_chunks_acknowledged = true;
                        if session.fall_teleport_confirmed {
                            session.limbo.start_check_after_client_ready();
                            info!("{} loaded Limbo chunks; starting three-second fall", session.player_name.as_deref().unwrap_or("unknown"));
                            send_limbo_title(client, session, &server.limbo_texts.render(&server.limbo_texts.checking_title), &server.limbo_texts.render(&server.limbo_texts.checking_subtitle)).await?;
                            send_client_body(
                                client,
                                session,
                                chat_packet(
                                    session.protocol_number,
                                    protocol.chat_clientbound(),
                                    &server.limbo_texts.render(&server.limbo_texts.checking_chat),
                                )?,
                            ).await?;
                        }
                        continue;
                    }

                    if session.limbo.phase == LimboPhase::Falling {
                        if protocol.is_movement_serverbound(id) {
                            session.limbo.note_movement_packet();
                        }
                        if id == protocol.move_ground_serverbound()
                            && view.first().is_some_and(|flags| flags & 1 != 0)
                        {
                            let reason = server.limbo_texts.render(&server.limbo_texts.bot_detected);
                            send_client_body(client, session, chat_packet(session.protocol_number, protocol.chat_clientbound(), &reason)?).await?;
                            return Ok(());
                        }
                        if let Some(movement) = protocol.read_movement(id, &mut view)? {
                            if matches!(session.limbo.observe_movement(movement), crate::limbo::FallResult::Invalid) {
                                let reason = server.limbo_texts.render(&server.limbo_texts.bot_detected);
                                send_client_body(client, session, chat_packet(session.protocol_number, protocol.chat_clientbound(), &reason)?).await?;
                                return Ok(());
                            }
                        }
                        continue;
                    }

                }
            }
            _ = tick.tick() => {
                if session.limbo.phase == LimboPhase::Falling
                    && session.fall_teleport_confirmed
                    && (session.protocol_number != 764 || session.limbo_chunks_acknowledged) {
                    if session.limbo.movement_timed_out() {
                        warn!("Limbo movement timed out for {} (protocol {})", session.player_name.as_deref().unwrap_or("unknown"), session.protocol_number);
                        let reason = server.limbo_texts.render(&server.limbo_texts.bot_detected);
                        send_client_body(client, session, chat_packet(session.protocol_number, protocol.chat_clientbound(), &reason)?).await?;
                        return Ok(());
                    }
                    if session.limbo.finish_fall_if_ready() {
                        if session.captcha_enabled {
                            begin_captcha(client, session, protocol, Some(&server.limbo_texts)).await?;
                        } else {
                            if !session.client_settings_seen || !session.client_brand_seen || !session.client_loaded_seen {
                                let reason = server.limbo_texts.render(&server.limbo_texts.bot_detected);
                                send_client_body(client, session, chat_packet(session.protocol_number, protocol.chat_clientbound(), &reason)?).await?;
                                return Ok(());
                            }
                            session.limbo.phase = LimboPhase::Passed;
                            let name = session.player_name.clone().unwrap_or_default();
                            let uuid = session.player_uuid.ok_or_else(|| anyhow::anyhow!("Limbo player UUID is missing"))?;
                            if let Some(mut backend) = connect_backend(server.clone(), session, client, &name, &uuid).await? {
                                let relay = run_relay(client, &mut backend, session).await;
                                server.active_players.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                                return relay;
                            }
                            return Ok(());
                        }
                    } else {
                        let level = session.limbo.progress().min(100);
                        let progress = (session.limbo.started_elapsed().as_secs_f32() / 3.0)
                            .clamp(0.0, 1.0);
                        send_client_body(
                            client,
                            session,
                            experience_packet(
                                session.protocol_number,
                                protocol.experience_clientbound(),
                                progress,
                                i32::from(level),
                            ),
                        )
                        .await?;
                    }
                } else if session.limbo.phase == LimboPhase::Captcha {
                    if session.limbo.expired() {
                        warn!("Limbo CAPTCHA timed out for {}", session.player_name.as_deref().unwrap_or("unknown"));
                        let timeout = server.limbo_texts.render(&server.limbo_texts.timeout);
                        send_client_body(client, session, chat_packet(session.protocol_number, protocol.chat_clientbound(), &timeout)?).await?;
                        return Ok(());
                    }
                }
            }
        }
    }
    Ok(())
}

struct LimboOnlineGuard(Arc<ProxyServer>);

impl Drop for LimboOnlineGuard {
    fn drop(&mut self) {
        self.0
            .active_limbo_players
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
}

async fn begin_captcha(
    client: &mut TcpStream,
    session: &mut ClientSession,
    protocol: ProtocolPackets,
    texts: Option<&crate::limbo::texts::LimboTexts>,
) -> anyhow::Result<()> {
    if !session.captcha_enabled {
        session.limbo.phase = LimboPhase::Passed;
        return Ok(());
    }
    let default_texts = crate::limbo::texts::LimboTexts::default();
    let texts = texts.unwrap_or(&default_texts);
    if matches!(session.protocol_number, 764 | 774) {
        let mut clear = BytesMut::new();
        VarInt::write(&mut clear, if session.protocol_number == 774 { 0x0E } else { 0x0F });
        clear.put_u8(1);
        send_client_body(client, session, clear).await?;
    }
    send_client_body(
        client,
        session,
        experience_packet(
            session.protocol_number,
            protocol.experience_clientbound(),
            1.0,
            100,
        ),
    )
        .await?;
    let mut position = session
        .limbo
        .origin
        .ok_or_else(|| anyhow::anyhow!("limbo origin is missing"))?;
    position.x = 12.0;
    position.y = if session.protocol_number == 754 { 220.0 } else { 400.0 };
    position.z = -12.0;
    position.yaw = 0.0;
    position.pitch = 0.0;
    let frame_facing = 2;
    const CAPTCHA_SHIFT_X: i32 = 0;
    const CAPTCHA_SHIFT_Y: i32 = 0;
    const CAPTCHA_SHIFT_Z: i32 = -2;
    let wall_block_z = -8 + CAPTCHA_SHIFT_Z; 
    let first_block_x = (position.x - 2.0).floor() as i32 + CAPTCHA_SHIFT_X;
    let first_block_y = position.y.floor() as i32 + CAPTCHA_SHIFT_Y;
    session.limbo.origin = Some(position);
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
            protocol.position_clientbound(),
            position,
        ),
    )
        .await?;
    send_client_body(
        client,
        session,
        ability_packet(protocol.abilities_clientbound(), 6, 0.0, 0.1),
    )
        .await?;
    if matches!(session.protocol_number, 754 | 764 | 774) {
        send_client_body(
            client,
            session,
            update_view_position_packet(
                session.protocol_number,
                (position.x.floor() as i32).div_euclid(16),
                (position.z.floor() as i32).div_euclid(16),
            )?,
        )
            .await?;
        send_self_visible(client, session).await?;
        for row in 0..3 {
            for col in 0..4 {
                let map_id = row * 4 + col;
                let entity_id = 10 + map_id as i32;
                let packets = if session.protocol_number == 754 {
                    framed_captcha_entity_packets_754(
                        entity_id,
                        map_id as i32,
                        first_block_x + col as i32,
                        first_block_y + row as i32,
                        wall_block_z,
                        frame_facing,
                    )?
                } else if session.protocol_number == 774 {
                    framed_captcha_entity_packets_774(
                        entity_id,
                        map_id as i32,
                        first_block_x + col as i32,
                        first_block_y + row as i32,
                        wall_block_z,
                        frame_facing,
                    )?
                } else {
                    framed_captcha_entity_packets_764(
                    entity_id,
                    map_id as i32,
                    first_block_x + col as i32,
                    first_block_y + row as i32,
                    wall_block_z,
                    frame_facing,
                    )?
                };
                for packet in packets {
                    send_client_body(client, session, packet).await?;
                }
            }
        }
        send_captcha_maps(client, session).await?;
        if !texts.captcha_title.is_empty() || !texts.captcha_subtitle.is_empty() {
            send_limbo_title(
                client,
                session,
                &texts.render(&texts.captcha_title),
                &texts.render(&texts.captcha_subtitle),
            )
                .await?;
        }
        send_client_body(
            client,
            session,
            chat_packet(
                session.protocol_number,
                protocol.chat_clientbound(),
                &texts.render(&texts.captcha_chat),
            )?,
        )
            .await?;
    }
    info!(
        "Sent Limbo CAPTCHA 4x3 in front of {} at blocks x={}..{}, y={}..{}, z={}; player at ({:.1}, {:.1}, {:.1})",
        session.player_name.as_deref().unwrap_or("unknown"),
        first_block_x,
        first_block_x + 3,
        first_block_y,
        first_block_y + 2,
        wall_block_z,
        position.x,
        position.y,
        position.z
    );
    Ok(())
}
pub(crate) async fn begin_captcha_for_relay(
    client: &mut TcpStream,
    session: &mut ClientSession,
    protocol: ProtocolPackets,
) -> anyhow::Result<()> {
    begin_captcha(client, session, protocol, None).await
}
async fn send_self_visible(
    client: &mut TcpStream,
    session: &ClientSession,
) -> anyhow::Result<()> {
    if session.protocol_number != 764 {
        return Ok(());
    }
    if let (Some(uuid), Some(name)) = (session.player_uuid.as_ref(), session.player_name.as_deref())
    {
        send_client_body(client, session, player_info_add_self_764(uuid, name)).await?;
    }
    send_client_body(client, session, self_visible_metadata_764(1)).await
}

async fn send_captcha_maps(
    client: &mut TcpStream,
    session: &ClientSession,
) -> anyhow::Result<()> {
    anyhow::ensure!(matches!(session.protocol_number, 754 | 764 | 774), "framed Limbo CAPTCHA is unsupported for protocol {}", session.protocol_number);
    let salt = Uuid::new_v4().as_u128() as u64;
    let maps = captcha::render_captcha(session.limbo.answer(), &session.captcha_dir, salt, session.captcha_background_count)?;
    anyhow::ensure!(
        maps.len() == 12,
        "4x3 CAPTCHA renderer produced {} maps",
        maps.len()
    );
    for (map_id, pixels) in maps.iter().enumerate() {
        let packet = if session.protocol_number == 754 {
            map_data_packet_754(map_id as i32, pixels)
        } else if session.protocol_number == 774 {
            map_data_packet_774(map_id as i32, pixels)
        } else {
            map_data_packet_764(map_id as i32, pixels)
        };
        send_client_body(client, session, packet).await?;
    }
    Ok(())
}

async fn send_limbo_title(
    client: &mut TcpStream,
    session: &mut ClientSession,
    title: &str,
    subtitle: &str,
) -> anyhow::Result<()> {
    let (title_packet_id, subtitle_packet_id, times_packet_id) = match session.protocol_number {
        764 => (0x61, 0x5F, 0x62),
        774 => (0x70, 0x6E, 0x71),
        _ => return Ok(()),
    };
    let mut times = BytesMut::new();
    VarInt::write(&mut times, times_packet_id);
    times.put_i32(3);
    times.put_i32(1200);
    times.put_i32(8);
    send_client_body(client, session, times).await?;

    let mut title_body = BytesMut::new();
    VarInt::write(&mut title_body, title_packet_id);
    write_title_component(&mut title_body, title)?;
    send_client_body(client, session, title_body).await?;

    let mut subtitle_body = BytesMut::new();
    VarInt::write(&mut subtitle_body, subtitle_packet_id);
    write_title_component(&mut subtitle_body, subtitle)?;
    send_client_body(client, session, subtitle_body).await
}

fn write_title_component(body: &mut BytesMut, text: &str) -> anyhow::Result<()> {
    let component = serde_json::to_string(&serde_json::json!({"text": text}))?;
    ProtocolUtils::write_string(body, &component);
    Ok(())
}
