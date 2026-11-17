use crate::protocol::packet::*;

use super::{PacketIds1_19, PacketIds1_20_2};

pub struct PacketIds1_21_11;
impl PacketIds1_21_11 {
    pub fn handshake(_: HandshakePacket) -> i32 {
        0x00
    }
    pub fn status(p: StatusPacket) -> i32 {
        PacketIds1_19::status(p)
    }
    pub fn login(p: LoginPacket) -> i32 {
        match p {
            LoginPacket::StartServerbound => 0x00,
            LoginPacket::EncryptionResponseServerbound => 0x01,
            LoginPacket::PluginResponseServerbound => 0x02,
            LoginPacket::LoginAcknowledgedServerbound => 0x03,
            LoginPacket::DisconnectClientbound => 0x00,
            LoginPacket::EncryptionRequestClientbound => 0x01,
            LoginPacket::SuccessClientbound => 0x02,
            LoginPacket::SetCompressionClientbound => 0x03,
            LoginPacket::PluginRequestClientbound => 0x04,
            LoginPacket::CookieRequestClientbound => 0x05,
        }
    }
    pub fn configuration(p: ConfigurationPacket) -> i32 {
        PacketIds1_20_2::configuration(p)
    }
}
