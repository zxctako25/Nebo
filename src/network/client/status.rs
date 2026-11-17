use bytes::{Buf, BytesMut};
use std::sync::Arc;
use tokio::net::TcpStream;

use crate::network::session::ClientSession;
use crate::network::server::ProxyServer;
use crate::protocol::factory::PacketFactory;
use crate::protocol::io::VarInt;
use crate::protocol::packet::StatusPacket;

use super::util::write_frame;

pub async fn handle_status(
    server: &Arc<ProxyServer>,
    session: &mut ClientSession,
    stream: &mut TcpStream,
    mut payload: BytesMut,
) -> anyhow::Result<()> {
    let packet_id = VarInt::read(&mut payload)?;
    if packet_id == session.protocol.status_id(StatusPacket::RequestServerbound) {
        let favicon = server.motd.favicon();
        let online = if server.config.show_online_count {
            let backend_players = server
                .active_players
                .load(std::sync::atomic::Ordering::Relaxed)
                .max(0);
            let limbo_players = server
                .active_limbo_players
                .load(std::sync::atomic::Ordering::Relaxed)
                .max(0);
            backend_players.saturating_add(limbo_players)
        } else {
            0
        };
        let mut response = serde_json::json!({
            "version": { "name": session.protocol.display_name(), "protocol": session.protocol_number },
            "players": { "max": server.config.max_players, "online": online },
            "description": server.motd.description()
        });
        if let Some(favicon) = favicon {
            response["favicon"] = serde_json::Value::String(favicon);
        }
        let json = serde_json::to_string(&response)?;

        let body = PacketFactory::status_response(&*session.protocol, &json);
        write_frame(stream, session, body).await?;
    } else if packet_id == session.protocol.status_id(StatusPacket::PingServerbound) {
        if payload.len() != 8 {
            anyhow::bail!("Invalid status ping payload length: {}", payload.len());
        }
        let payload_long = i64::from_be_bytes(payload[..8].try_into()?);
        let body = PacketFactory::status_pong(&*session.protocol, payload_long);
        write_frame(stream, session, body).await?;
    }
    Ok(())
}
