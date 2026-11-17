use bytes::{Buf, BufMut, BytesMut};
use md5::{Digest, Md5};
use std::io;
use uuid::Uuid;

use super::data::*;
use super::io::{ProtocolUtils, VarInt};
use super::packet::*;

pub mod v1_16_5;
pub mod v1_17;
pub mod v1_18;
pub mod v1_19;
pub mod v1_19_3;
pub mod v1_20;
pub mod v1_20_2;
pub mod v1_21_11;

pub use v1_16_5::Protocol1_16_5;
pub use v1_17::Protocol1_17;
pub use v1_18::Protocol1_18;
pub use v1_19::Protocol1_19;
pub use v1_19_3::Protocol1_19_3;
pub use v1_20::Protocol1_20;
pub use v1_20_2::Protocol1_20_2;
pub use v1_21_11::Protocol1_21_11;

pub(crate) fn read_login_start_optional_uuid(buf: &mut BytesMut) -> io::Result<LoginStartData> {
    let name = ProtocolUtils::read_string(buf)?;
    let uuid = if buf.has_remaining() && buf.get_u8() != 0 {
        Some(ProtocolUtils::read_uuid(buf)?)
    } else {
        None
    };
    Ok(LoginStartData { name, uuid })
}

pub(crate) fn read_login_start_required_uuid(buf: &mut BytesMut) -> io::Result<LoginStartData> {
    Ok(LoginStartData {
        name: ProtocolUtils::read_string(buf)?,
        uuid: Some(ProtocolUtils::read_uuid(buf)?),
    })
}

pub(crate) fn write_login_start_optional_uuid(buf: &mut BytesMut, name: &str) {
    ProtocolUtils::write_string(buf, name);
    buf.put_u8(0);
}

pub(crate) fn write_login_start_required_uuid(buf: &mut BytesMut, name: &str, uuid: &Uuid) {
    ProtocolUtils::write_string(buf, name);
    ProtocolUtils::write_uuid(buf, uuid);
}

pub trait MinecraftProtocol: Send + Sync {
    fn protocol(&self) -> i32;
    fn display_name(&self) -> &'static str;
    fn has_configuration_state(&self) -> bool;
    fn supports_set_compression(&self) -> bool;

    fn read_handshake(&self, buf: &mut BytesMut) -> io::Result<HandshakeData>;
    fn write_handshake(
        &self,
        buf: &mut BytesMut,
        protocol: i32,
        host: &str,
        port: u16,
        next: ConnectionState,
    );
    fn read_login_start(&self, buf: &mut BytesMut) -> io::Result<LoginStartData>;
    fn write_login_start(&self, buf: &mut BytesMut, name: &str, uuid: &Uuid);
    fn read_login_success(&self, buf: &mut BytesMut) -> io::Result<LoginSuccessData>;
    fn read_plugin_response(&self, buf: &mut BytesMut) -> io::Result<PluginResponseData>;
    fn write_plugin_response(
        &self,
        buf: &mut BytesMut,
        message_id: i32,
        ok: bool,
        data: Option<&[u8]>,
    );
    fn read_set_compression(&self, buf: &mut BytesMut) -> io::Result<i32>;
    fn write_set_compression(&self, buf: &mut BytesMut, threshold: i32);
    fn read_disconnect(&self, buf: &mut BytesMut) -> io::Result<String>;
    fn read_login_acknowledged(&self, buf: &mut BytesMut) -> io::Result<()>;

    fn offline_uuid(&self, name: &str) -> Uuid;

    fn handshake_id(&self, p: HandshakePacket) -> i32;
    fn status_id(&self, p: StatusPacket) -> i32;
    fn login_id(&self, p: LoginPacket) -> i32;
    fn configuration_id(&self, p: ConfigurationPacket) -> i32;
}

#[allow(dead_code)]
pub(crate) fn offline_uuid_from_name(name: &str) -> Uuid {
    let mut hasher = Md5::new();
    hasher.update(format!("OfflinePlayer:{}", name).as_bytes());
    let hash = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

#[allow(unused_imports)]
use crate::protocol::io::VarInt as _VarIntReexport;
