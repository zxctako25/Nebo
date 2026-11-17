use bytes::BytesMut;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::codec::frame::FrameEncoder;
use crate::network::session::ClientSession;

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

pub async fn send_client_body(
    client: &mut TcpStream,
    session: &ClientSession,
    body: BytesMut,
) -> anyhow::Result<()> {
    let encoded = session.compression_encoder.encode(&body)?;
    client.write_all(&FrameEncoder::encode(&encoded)).await?;
    Ok(())
}
