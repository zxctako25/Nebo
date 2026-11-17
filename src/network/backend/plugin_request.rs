use bytes::{Buf, BytesMut};
use log::info;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::codec::compression::CompressionEncoder;
use crate::codec::frame::FrameEncoder;
use crate::forwarding::velocity;
use crate::network::session::ClientSession;
use crate::network::server::ProxyServer;
use crate::protocol::constants::*;
use crate::protocol::factory::PacketFactory;
use crate::protocol::io::{ProtocolUtils, VarInt};
use crate::protocol::packet::*;
use crate::protocol::versions::MinecraftProtocol;

pub async fn handle_plugin_request(
    server: Arc<ProxyServer>,
    session: &mut ClientSession,
    protocol: &dyn MinecraftProtocol,
    backend_compression_encoder: &mut CompressionEncoder,
    backend: &mut TcpStream,
    mut payload_read: BytesMut,
) -> anyhow::Result<()> {
    let message_id = VarInt::read(&mut payload_read)?;
    let channel = ProtocolUtils::read_string(&mut payload_read)?;
    let requested_version = if payload_read.has_remaining() {
        VarInt::read(&mut payload_read)?
    } else {
        0
    };

    if velocity::CHANNEL == channel {
        if !server.forwarding.modern_enabled {
            anyhow::bail!("Backend requested velocity but forwarding disabled");
        }
        let data = server.forwarding.create_forwarding_data(
            requested_version,
            &session.client_address,
            session.player_uuid.as_ref().expect("uuid set before backend connect"),
            session.player_name.as_deref().expect("name set before backend connect"),
        );
        let resp = match &data {
            Some(d) => PacketFactory::plugin_response(protocol, message_id, true, Some(d)),
            None => PacketFactory::plugin_response(protocol, message_id, false, None),
        };
        let enc = backend_compression_encoder.encode(&resp)?;
        let framed = FrameEncoder::encode(&enc);
        backend.write_all(&framed).await?;
        info!(
            "Velocity forwarding sent (v={}, {} bytes)",
            requested_version.min(VELOCITY_MODERN_MAX),
            data.map(|d| d.len()).unwrap_or(0)
        );
    } else {
        let resp = PacketFactory::plugin_response(protocol, message_id, false, None);
        let enc = backend_compression_encoder.encode(&resp)?;
        let framed = FrameEncoder::encode(&enc);
        backend.write_all(&framed).await?;
    }
    Ok(())
}
