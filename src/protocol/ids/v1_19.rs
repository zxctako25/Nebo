use crate::protocol::packet::*;

pub struct PacketIds1_19;
impl PacketIds1_19 {
    pub fn handshake(_: HandshakePacket) -> i32 {
        0x00
    }
    pub fn status(p: StatusPacket) -> i32 {
        match p {
            StatusPacket::RequestServerbound => 0x00,
            StatusPacket::PingServerbound => 0x01,
            StatusPacket::ResponseClientbound => 0x00,
            StatusPacket::PongClientbound => 0x01,
        }
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
            LoginPacket::CookieRequestClientbound => -1,
        }
    }
}
