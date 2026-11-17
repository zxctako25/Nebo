use bytes::{Buf, BufMut, BytesMut};
use std::io;
use uuid::Uuid;

use crate::protocol::data::*;
use crate::protocol::ids::PacketIds1_16_5;
use crate::protocol::io::{ProtocolUtils, VarInt};
use crate::protocol::packet::*;

use super::{offline_uuid_from_name, MinecraftProtocol};

pub struct Protocol1_16_5;

impl MinecraftProtocol for Protocol1_16_5 {
    fn protocol(&self) -> i32 {
        754
    }
    fn display_name(&self) -> &'static str {
        "1.16.5"
    }
    fn has_configuration_state(&self) -> bool {
        false
    }
    fn supports_set_compression(&self) -> bool {
        true
    }

    fn read_handshake(&self, buf: &mut BytesMut) -> io::Result<HandshakeData> {
        let protocol = VarInt::read(buf)?;
        let host = ProtocolUtils::read_string(buf)?;
        let port = buf.get_u16();
        let ns = VarInt::read(buf)?;
        let next_state = ConnectionState::by_id(ns).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid nextState: {}", ns),
            )
        })?;
        Ok(HandshakeData {
            protocol,
            host,
            port,
            next_state,
        })
    }

    fn write_handshake(
        &self,
        buf: &mut BytesMut,
        protocol: i32,
        host: &str,
        port: u16,
        next: ConnectionState,
    ) {
        VarInt::write(buf, protocol);
        ProtocolUtils::write_string(buf, host);
        buf.put_u16(port);
        VarInt::write(buf, next.id());
    }

    fn read_login_start(&self, buf: &mut BytesMut) -> io::Result<LoginStartData> {
        Ok(LoginStartData {
            name: ProtocolUtils::read_string(buf)?,
            uuid: None,
        })
    }

    fn write_login_start(&self, buf: &mut BytesMut, name: &str, _uuid: &Uuid) {
        ProtocolUtils::write_string(buf, name);
    }

    fn read_login_success(&self, buf: &mut BytesMut) -> io::Result<LoginSuccessData> {
        let uuid = ProtocolUtils::read_uuid(buf)?;
        let name = ProtocolUtils::read_string(buf)?;
        Ok(LoginSuccessData { uuid, name })
    }

    fn read_plugin_response(&self, buf: &mut BytesMut) -> io::Result<PluginResponseData> {
        let message_id = VarInt::read(buf)?;
        let successful = buf.get_u8() != 0;
        let data = if successful && buf.has_remaining() {
            Some(ProtocolUtils::read_rest(buf))
        } else {
            None
        };
        Ok(PluginResponseData {
            message_id,
            successful,
            data,
        })
    }

    fn write_plugin_response(
        &self,
        buf: &mut BytesMut,
        message_id: i32,
        ok: bool,
        data: Option<&[u8]>,
    ) {
        VarInt::write(buf, message_id);
        buf.put_u8(if ok { 1 } else { 0 });
        if ok {
            if let Some(d) = data {
                buf.put_slice(d);
            }
        }
    }

    fn read_set_compression(&self, buf: &mut BytesMut) -> io::Result<i32> {
        VarInt::read(buf)
    }
    fn write_set_compression(&self, buf: &mut BytesMut, threshold: i32) {
        VarInt::write(buf, threshold);
    }

    fn read_disconnect(&self, buf: &mut BytesMut) -> io::Result<String> {
        ProtocolUtils::read_string(buf)
    }

    fn read_login_acknowledged(&self, _buf: &mut BytesMut) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Login Acknowledged not present",
        ))
    }

    fn offline_uuid(&self, name: &str) -> Uuid {
        offline_uuid_from_name(name)
    }

    fn handshake_id(&self, p: HandshakePacket) -> i32 {
        PacketIds1_16_5::handshake(p)
    }
    fn status_id(&self, p: StatusPacket) -> i32 {
        PacketIds1_16_5::status(p)
    }
    fn login_id(&self, p: LoginPacket) -> i32 {
        PacketIds1_16_5::login(p)
    }
    fn configuration_id(&self, _: ConfigurationPacket) -> i32 {
        -1
    }
}
