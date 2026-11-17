use bytes::BytesMut;
use uuid::Uuid;

use super::io::{ProtocolUtils, VarInt};
use super::packet::*;
use super::versions::MinecraftProtocol;

pub struct PacketFactory;

impl PacketFactory {
    pub fn handshake(
        protocol: &dyn MinecraftProtocol,
        ver: i32,
        host: &str,
        port: u16,
        next: ConnectionState,
    ) -> BytesMut {
        let mut b = BytesMut::new();
        VarInt::write(&mut b, protocol.handshake_id(HandshakePacket::Serverbound));
        protocol.write_handshake(&mut b, ver, host, port, next);
        b
    }

    pub fn status_response(protocol: &dyn MinecraftProtocol, json: &str) -> BytesMut {
        let mut b = BytesMut::new();
        VarInt::write(
            &mut b,
            protocol.status_id(StatusPacket::ResponseClientbound),
        );
        ProtocolUtils::write_string(&mut b, json);
        b
    }

    pub fn status_pong(protocol: &dyn MinecraftProtocol, payload: i64) -> BytesMut {
        let mut b = BytesMut::new();
        VarInt::write(&mut b, protocol.status_id(StatusPacket::PongClientbound));
        b.extend_from_slice(&payload.to_be_bytes());
        b
    }

    pub fn login_disconnect(protocol: &dyn MinecraftProtocol, json: &str) -> BytesMut {
        let mut b = BytesMut::new();
        VarInt::write(
            &mut b,
            protocol.login_id(LoginPacket::DisconnectClientbound),
        );
        ProtocolUtils::write_string(&mut b, json);
        b
    }

    pub fn set_compression(protocol: &dyn MinecraftProtocol, threshold: i32) -> BytesMut {
        let mut b = BytesMut::new();
        VarInt::write(
            &mut b,
            protocol.login_id(LoginPacket::SetCompressionClientbound),
        );
        protocol.write_set_compression(&mut b, threshold);
        b
    }

    pub fn login_start(protocol: &dyn MinecraftProtocol, name: &str, uuid: &Uuid) -> BytesMut {
        let mut b = BytesMut::new();
        VarInt::write(&mut b, protocol.login_id(LoginPacket::StartServerbound));
        protocol.write_login_start(&mut b, name, uuid);
        b
    }

    pub fn login_success(
        protocol: &dyn MinecraftProtocol,
        protocol_number: i32,
        name: &str,
        uuid: &Uuid,
    ) -> BytesMut {
        let mut b = BytesMut::new();
        VarInt::write(&mut b, protocol.login_id(LoginPacket::SuccessClientbound));
        ProtocolUtils::write_uuid(&mut b, uuid);
        ProtocolUtils::write_string(&mut b, name);
        if protocol.protocol() >= 759 {
            VarInt::write(&mut b, 0); 
        }
        if matches!(protocol_number, 766 | 767) {
            b.extend_from_slice(&[0]); 
        }
        b
    }

    pub fn login_acknowledged(protocol: &dyn MinecraftProtocol) -> BytesMut {
        let mut b = BytesMut::new();
        VarInt::write(
            &mut b,
            protocol.login_id(LoginPacket::LoginAcknowledgedServerbound),
        );
        b
    }

    pub fn plugin_response(
        protocol: &dyn MinecraftProtocol,
        message_id: i32,
        ok: bool,
        data: Option<&[u8]>,
    ) -> BytesMut {
        let mut b = BytesMut::new();
        VarInt::write(
            &mut b,
            protocol.login_id(LoginPacket::PluginResponseServerbound),
        );
        protocol.write_plugin_response(&mut b, message_id, ok, data);
        b
    }
}
