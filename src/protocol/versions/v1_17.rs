use bytes::BytesMut;
use std::io;
use uuid::Uuid;

use crate::protocol::data::*;
use crate::protocol::ids::PacketIds1_17;
use crate::protocol::io::ProtocolUtils;
use crate::protocol::packet::*;

use super::{offline_uuid_from_name, Protocol1_16_5, MinecraftProtocol};

pub struct Protocol1_17;

impl MinecraftProtocol for Protocol1_17 {
    fn protocol(&self) -> i32 {
        755
    }
    fn display_name(&self) -> &'static str {
        "1.17.x"
    }
    fn has_configuration_state(&self) -> bool {
        false
    }
    fn supports_set_compression(&self) -> bool {
        true
    }
    fn read_handshake(&self, buf: &mut BytesMut) -> io::Result<HandshakeData> {
        Protocol1_16_5.read_handshake(buf)
    }
    fn write_handshake(&self, buf: &mut BytesMut, p: i32, h: &str, po: u16, n: ConnectionState) {
        Protocol1_16_5.write_handshake(buf, p, h, po, n)
    }
    fn read_login_start(&self, buf: &mut BytesMut) -> io::Result<LoginStartData> {
        Protocol1_16_5.read_login_start(buf)
    }
    fn write_login_start(&self, buf: &mut BytesMut, name: &str, _uuid: &Uuid) {
        ProtocolUtils::write_string(buf, name);
    }
    fn read_login_success(&self, buf: &mut BytesMut) -> io::Result<LoginSuccessData> {
        Protocol1_16_5.read_login_success(buf)
    }
    fn read_plugin_response(&self, buf: &mut BytesMut) -> io::Result<PluginResponseData> {
        Protocol1_16_5.read_plugin_response(buf)
    }
    fn write_plugin_response(&self, buf: &mut BytesMut, m: i32, o: bool, d: Option<&[u8]>) {
        Protocol1_16_5.write_plugin_response(buf, m, o, d)
    }
    fn read_set_compression(&self, buf: &mut BytesMut) -> io::Result<i32> {
        Protocol1_16_5.read_set_compression(buf)
    }
    fn write_set_compression(&self, buf: &mut BytesMut, t: i32) {
        Protocol1_16_5.write_set_compression(buf, t)
    }
    fn read_disconnect(&self, buf: &mut BytesMut) -> io::Result<String> {
        Protocol1_16_5.read_disconnect(buf)
    }
    fn read_login_acknowledged(&self, _: &mut BytesMut) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "unsupported"))
    }
    fn offline_uuid(&self, name: &str) -> Uuid {
        offline_uuid_from_name(name)
    }
    fn handshake_id(&self, p: HandshakePacket) -> i32 {
        PacketIds1_17::handshake(p)
    }
    fn status_id(&self, p: StatusPacket) -> i32 {
        PacketIds1_17::status(p)
    }
    fn login_id(&self, p: LoginPacket) -> i32 {
        PacketIds1_17::login(p)
    }
    fn configuration_id(&self, _: ConfigurationPacket) -> i32 {
        -1
    }
}
