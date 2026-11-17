use bytes::{Buf, BytesMut};
use log::info;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::codec::frame::FrameEncoder;
use crate::limbo::LimboPhase;
use crate::network::relay::RelayState;
use crate::network::session::ClientSession;
use crate::protocol::io::VarInt;
use crate::protocol::packet::ConnectionState;

pub(crate) async fn handle_backend_frame(
    client: &mut TcpStream,
    _backend: &mut TcpStream,
    session: &mut ClientSession,
    state: &mut RelayState,
    payload: BytesMut,
) -> anyhow::Result<bool> {
    let protocol = state.protocol;
    let mut view = payload.clone();
    let id = VarInt::read(&mut view)?;
    if id == protocol.abilities_clientbound() {
        state.backend_abilities = Some(payload.clone());
    }
    if id == protocol.experience_clientbound() {
        state.backend_experience = Some(payload.clone());
    }
    if session.state == ConnectionState::Configuration
        && id == protocol.finish_configuration_clientbound()
    {
        state.backend_finished_configuration = true;
    }
    if !state.client_finished_configuration || !state.backend_finished_configuration {
        let encoded = session.compression_encoder.encode(&payload)?;
        client.write_all(&FrameEncoder::encode(&encoded)).await?;
        return Ok(true);
    }
    session.state = ConnectionState::Play;
    if id == protocol.position_clientbound() && session.limbo.phase == LimboPhase::WaitingForJoin {
        let origin = protocol.read_position(&mut view)?;
        let mut high = origin;
        high.y = 1000.0;
        session.limbo.start_fall(origin, high.y);
        high.teleport_id = session.limbo.origin_teleport_id.wrapping_add(1);
        info!(
            "Limbo check started for {} (protocol {})",
            session.player_name.as_deref().unwrap_or("unknown"),
            session.protocol_number
        );
        crate::network::client::util::send_client_body(
            client,
            session,
            crate::limbo::packets::position_packet(session.protocol_number, id, high),
        )
            .await?;
        return Ok(true);
    }
    if matches!(
        session.limbo.phase,
        LimboPhase::Falling | LimboPhase::Captcha
    ) {
        if id == protocol.experience_clientbound()
            || id == protocol.position_clientbound()
            || id == protocol.abilities_clientbound()
        {
            return Ok(true);
        }
    }
    let encoded = session.compression_encoder.encode(&payload)?;
    client.write_all(&FrameEncoder::encode(&encoded)).await?;
    Ok(true)
}
