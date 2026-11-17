use crate::protocol::packet::*;

use super::{PacketIds1_19, PacketIds1_20};

pub struct PacketIds1_20_2;
impl PacketIds1_20_2 {
    pub fn handshake(_: HandshakePacket) -> i32 {
        0x00
    }
    pub fn status(p: StatusPacket) -> i32 {
        PacketIds1_19::status(p)
    }
    pub fn login(p: LoginPacket) -> i32 {
        PacketIds1_20::login(p)
    }
    pub fn configuration(p: ConfigurationPacket) -> i32 {
        match p {
            ConfigurationPacket::ClientInformationServerbound => 0x00,
            ConfigurationPacket::PluginMessageServerbound => 0x01,
            ConfigurationPacket::FinishConfigurationServerbound => 0x02,
            ConfigurationPacket::KeepAliveServerbound => 0x03,
            ConfigurationPacket::PongServerbound => 0x04,
            ConfigurationPacket::ResourcePackResponseServerbound => 0x05,
            ConfigurationPacket::PluginMessageClientbound => 0x00,
            ConfigurationPacket::DisconnectClientbound => 0x01,
            ConfigurationPacket::FinishConfigurationClientbound => 0x02,
            ConfigurationPacket::KeepAliveClientbound => 0x03,
            ConfigurationPacket::PingClientbound => 0x04,
            ConfigurationPacket::RegistryDataClientbound => 0x05,
            ConfigurationPacket::ResourcePackClientbound => 0x06,
            ConfigurationPacket::UpdateTagsClientbound => 0x07,
        }
    }
}
