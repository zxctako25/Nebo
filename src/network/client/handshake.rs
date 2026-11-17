use bytes::{Buf, BytesMut};
use log::info;
use std::sync::Arc;
use tokio::net::TcpStream;

use crate::network::session::ClientSession;
use crate::network::server::ProxyServer;
use crate::protocol::factory::PacketFactory;
use crate::protocol::io::VarInt;
use crate::protocol::packet::HandshakePacket;
use crate::protocol::registry::ProtocolRegistry;
use crate::util::chat::ChatComponent;

use super::util::write_frame;

pub async fn handle_handshake(
    server: &Arc<ProxyServer>,
    session: &mut ClientSession,
    stream: &mut TcpStream,
    mut payload: BytesMut,
) -> anyhow::Result<bool> {
    let packet_id = VarInt::read(&mut payload)?;
    if packet_id != session.protocol.handshake_id(HandshakePacket::Serverbound) {
        anyhow::bail!("Expected Handshake, got 0x{:x}", packet_id);
    }
    let hs = session.protocol.read_handshake(&mut payload)?;
    if !ProtocolRegistry::is_supported(hs.protocol) {
        let reason = ChatComponent::text(&server.config.unsupported_version_message);
        let body = PacketFactory::login_disconnect(&*session.protocol, &reason);
        write_frame(stream, session, body).await?;
        return Ok(false);
    }
    session.protocol = ProtocolRegistry::by_protocol(hs.protocol);
    session.protocol_number = hs.protocol;
    session.client_loaded_seen =
        crate::limbo::ProtocolPackets::new(hs.protocol).player_loaded_serverbound() < 0;
    session.state = hs.next_state;
    info!(
        "Handshake: {} ({}), nextState={:?}",
        hs.protocol,
        session.protocol.display_name(),
        session.state
    );
    Ok(true)
}
