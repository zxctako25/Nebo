use crate::protocol::packet::*;

use super::PacketIds1_16_5;

pub struct PacketIds1_17;
impl PacketIds1_17 {
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
