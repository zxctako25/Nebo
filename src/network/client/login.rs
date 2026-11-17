use bytes::{Buf, BytesMut};
use log::{info, warn};
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::Arc;
use tokio::net::TcpStream;

use crate::network::session::ClientSession;
use crate::network::server::ProxyServer;
use crate::protocol::factory::PacketFactory;
use crate::protocol::io::VarInt;
use crate::protocol::packet::{ConnectionState, LoginPacket};
use crate::security::anti_bot::BlockDecision;
use crate::util::chat::ChatComponent;

use super::{limbo_phase, util::write_frame};

static USERNAME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Za-z0-9_]{3,16}$").unwrap());

pub async fn handle_login(
    server: &Arc<ProxyServer>,
    session: &mut ClientSession,
    stream: &mut TcpStream,
    mut payload: BytesMut,
) -> anyhow::Result<Option<TcpStream>> {
    if let Ok(peer_ip) = session.client_address.parse() {
        info!(
            "JustDefender pre-login check for client IP {}",
            session.client_address
        );
        if let Some((checked_ip, decision)) = server.anti_bot.check_ip(peer_ip).await {
            if checked_ip != peer_ip {
                info!(
                    "Local client {} mapped to public egress IP {} for temporary geo test",
                    peer_ip, checked_ip
                );
            }
            let checked_ip_text = checked_ip.to_string();
            let message = match decision {
                BlockDecision::Country { name } => server.limbo_texts.anti_bot_country.replace("{country}", &name).replace("{ip}", &checked_ip_text),
                BlockDecision::Asn { number, organization } => server.limbo_texts.anti_bot_asn.replace("{organization}", &organization).replace("{asn}", &number.to_string()).replace("{ip}", &checked_ip_text),
                BlockDecision::Ip => server.limbo_texts.anti_bot_ip.replace("{ip}", &checked_ip_text),
            };
            let body =
                PacketFactory::login_disconnect(&*session.protocol, &ChatComponent::text(&message));
            write_frame(stream, session, body).await?;
            session.close_after_response = true;
            warn!(
                "Rejected client {} (checked as {}) by anti-bot rule",
                session.client_address, checked_ip
            );
            return Ok(None);
        }
    }
    let packet_id = VarInt::read(&mut payload)?;
    if packet_id != session.protocol.login_id(LoginPacket::StartServerbound) {
        anyhow::bail!("Expected Login Start, got 0x{:x}", packet_id);
    }
    let ls = session.protocol.read_login_start(&mut payload)?;
    let name = ls.name;
    if !USERNAME_RE.is_match(&name) {
        let reason = ChatComponent::text(&server.config.invalid_username_message);
        let body = PacketFactory::login_disconnect(&*session.protocol, &reason);
        write_frame(stream, session, body).await?;
        session.close_after_response = true;
        return Ok(None);
    }
    let uuid = session.protocol.offline_uuid(&name);
    info!("Login Start: {} (UUID: {})", name, uuid);

    if server.config.online_mode {
        let reason = ChatComponent::text(&server.config.online_mode_unsupported_message);
        let body = PacketFactory::login_disconnect(&*session.protocol, &reason);
        write_frame(stream, session, body).await?;
        session.close_after_response = true;
        return Ok(None);
    }

    session.player_name = Some(name.clone());
    session.player_uuid = Some(uuid);
    write_frame(
        stream,
        session,
        PacketFactory::login_success(&*session.protocol, session.protocol_number, &name, &uuid),
    )
        .await?;

    if !server.anti_bot.limbo_enabled {
        return crate::network::backend::connect_backend(
            server.clone(),
            session,
            stream,
            &name,
            &uuid,
        )
        .await;
    }

    if session.protocol_number >= 759 {
        session.state = ConnectionState::AwaitingLoginAck;
    } else {
        session.state = ConnectionState::Play;
        limbo_phase::begin_local_world(stream, session).await?;
    }
    Ok(None)
}
