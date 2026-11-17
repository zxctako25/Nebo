use bytes::BytesMut;
use log::info;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::codec::frame::FrameEncoder;
use crate::network::session::ClientSession;
use crate::protocol::factory::PacketFactory;

pub async fn write_frame(
    stream: &mut TcpStream,
    session: &ClientSession,
    body: BytesMut,
) -> anyhow::Result<()> {
    let payload = if session.compression_enabled {
        session.compression_encoder.encode(&body)?
    } else {
        body
    };
    let framed = FrameEncoder::encode(&payload);
    stream.write_all(&framed).await?;
    Ok(())
}

pub async fn enable_client_compression(
    session: &mut ClientSession,
    stream: &mut TcpStream,
    threshold: i32,
) -> anyhow::Result<()> {
    if session.compression_enabled {
        return Ok(());
    }
    if !session.protocol.supports_set_compression() {
        return Ok(());
    }
    session.compression_enabled = true;

    let body = PacketFactory::set_compression(&*session.protocol, threshold);
    let framed = FrameEncoder::encode(&body);
    stream.write_all(&framed).await?;

    session.compression_decoder.enable(threshold);
    session.compression_encoder.enable(threshold);
    info!("Client compression enabled (threshold={})", threshold);
    Ok(())
}
