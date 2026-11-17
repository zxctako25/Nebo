use bytes::{Buf, BytesMut};
use tokio::net::TcpStream;

use crate::limbo::ProtocolPackets;
use crate::network::session::ClientSession;
use crate::protocol::io::{ProtocolUtils, VarInt};

use super::limbo_phase::{begin_local_world, is_brand_message, preserve_setup_packet};

pub async fn handle_local_configuration(
    stream: &mut TcpStream,
    session: &mut ClientSession,
    mut payload: BytesMut,
) -> anyhow::Result<bool> {
    let protocol = ProtocolPackets::new(session.protocol_number);
    let full_packet = payload.clone();
    let id = VarInt::read(&mut payload)?;

    if id == protocol.client_information_configuration() {
        if !session.client_settings_seen {
            session.client_settings_seen = true;
            preserve_setup_packet(session, full_packet)?;
        }
        return Ok(false);
    }
    if id == protocol.plugin_message_serverbound_configuration() {
        let mut message = payload.clone();
        if is_brand_message(&mut message)? {
            if !session.client_brand_seen {
                session.client_brand_seen = true;
                preserve_setup_packet(session, full_packet)?;
            }
        }
        return Ok(false);
    }

    if session.local_config_packets_sent && id == protocol.finish_configuration_serverbound() {
        session.state = crate::protocol::packet::ConnectionState::Play;
        begin_local_world(stream, session).await?;
        return Ok(true);
    }
    Ok(false)
}
