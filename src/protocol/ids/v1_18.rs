use crate::protocol::packet::*;

use super::PacketIds1_16_5;

pub struct PacketIds1_18;
impl PacketIds1_18 {
    pub fn handshake(p: HandshakePacket) -> i32 {
        PacketIds1_16_5::handshake(p)
    }
    pub fn status(p: StatusPacket) -> i32 {
        PacketIds1_16_5::status(p)
    }
    pub fn login(p: LoginPacket) -> i32 {
        PacketIds1_16_5::login(p)
    }
}
