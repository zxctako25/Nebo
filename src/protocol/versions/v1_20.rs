use bytes::BytesMut;
use std::io;
use uuid::Uuid;

use crate::protocol::data::*;
use crate::protocol::ids::PacketIds1_20;
use crate::protocol::packet::*;

use super::{
    offline_uuid_from_name, read_login_start_optional_uuid, write_login_start_optional_uuid,
    Protocol1_16_5, MinecraftProtocol,
};

pub struct Protocol1_20;

impl MinecraftProtocol for Protocol1_20 {
    fn protocol(&self) -> i32 {
        763
    }
    fn display_name(&self) -> &'static str {
        "1.20/1.20.1"
    }
    fn has_configuration_state(&self) -> bool {
        false
    }
    fn supports_set_compression(&self) -> bool {
        true
    }
    fn read_handshake(&self, b: &mut BytesMut) -> io::Result<HandshakeData> {
        Protocol1_16_5.read_handshake(b)
    }
    fn write_handshake(&self, b: &mut BytesMut, p: i32, h: &str, po: u16, n: ConnectionState) {
        Protocol1_16_5.write_handshake(b, p, h, po, n)
    }
    fn read_login_start(&self, b: &mut BytesMut) -> io::Result<LoginStartData> {
        read_login_start_optional_uuid(b)
    }
    fn write_login_start(&self, b: &mut BytesMut, name: &str, uuid: &Uuid) {
        let _ = uuid;
        write_login_start_optional_uuid(b, name)
    }
    fn read_login_success(&self, b: &mut BytesMut) -> io::Result<LoginSuccessData> {
        Protocol1_16_5.read_login_success(b)
    }
    fn read_plugin_response(&self, b: &mut BytesMut) -> io::Result<PluginResponseData> {
        Protocol1_16_5.read_plugin_response(b)
    }
    fn write_plugin_response(&self, b: &mut BytesMut, m: i32, o: bool, d: Option<&[u8]>) {
        Protocol1_16_5.write_plugin_response(b, m, o, d)
    }
    fn read_set_compression(&self, b: &mut BytesMut) -> io::Result<i32> {
        Protocol1_16_5.read_set_compression(b)
    }
    fn write_set_compression(&self, b: &mut BytesMut, t: i32) {
        Protocol1_16_5.write_set_compression(b, t)
    }
    fn read_disconnect(&self, b: &mut BytesMut) -> io::Result<String> {
        Protocol1_16_5.read_disconnect(b)
    }
    fn read_login_acknowledged(&self, _: &mut BytesMut) -> io::Result<()> {
        Ok(())
    }
    fn offline_uuid(&self, n: &str) -> Uuid {
        offline_uuid_from_name(n)
    }
    fn handshake_id(&self, p: HandshakePacket) -> i32 {
        PacketIds1_20::handshake(p)
    }
    fn status_id(&self, p: StatusPacket) -> i32 {
        PacketIds1_20::status(p)
    }
    fn login_id(&self, p: LoginPacket) -> i32 {
        PacketIds1_20::login(p)
    }
    fn configuration_id(&self, _: ConfigurationPacket) -> i32 {
        -1
    }
}
