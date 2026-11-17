use bytes::{Buf, BytesMut};
use std::io;

use crate::protocol::io::VarInt;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub teleport_id: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct Movement {
    pub position: Position,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ProtocolPackets {
    protocol: i32,
}

impl ProtocolPackets {
    pub fn new(protocol: i32) -> Self {
        Self { protocol }
    }

    pub fn is_modern_position(self) -> bool {
        self.protocol >= 768
    }

    pub fn client_information_configuration(self) -> i32 {
        if self.protocol >= 764 {
            0x00
        } else {
            -1
        }
    }

    pub fn plugin_message_serverbound_configuration(self) -> i32 {
        match self.protocol {
            764..=765 => 0x01,
            766..=774 => 0x02,
            _ => -1,
        }
    }

    pub fn finish_configuration_serverbound(self) -> i32 {
        match self.protocol {
            764..=765 => 0x02,
            766..=774 => 0x03,
            _ => -1,
        }
    }
    pub fn configuration_acknowledged_play_serverbound(self) -> i32 {
        match self.protocol {
            764..=765 => 0x0B,
            766..=767 => 0x0C,
            768..=774 => 0x0E,
            _ => -1,
        }
    }

    pub fn finish_configuration_clientbound(self) -> i32 {
        match self.protocol {
            764..=765 => 0x02,
            766..=774 => 0x03,
            _ => -1,
        }
    }

    pub fn login_acknowledged_serverbound(self) -> i32 {
        if self.protocol >= 759 {
            0x03
        } else {
            -1
        }
    }

    pub fn start_configuration_clientbound(self) -> i32 {
        match self.protocol {
            764 => 0x65,
            765 => 0x67,
            766..=767 => 0x69,
            768..=769 => 0x70,
            770..=772 => 0x6F,
            773..=774 => 0x74,
            _ => -1,
        }
    }

    pub fn position_clientbound(self) -> i32 {
        match self.protocol {
            754 => 0x35,
            755..=758 => 0x38,
            759 => 0x36,
            760 => 0x39,
            761 => 0x38,
            762..=763 => 0x3C,
            764..=765 => 0x3E,
            766..=767 => 0x40,
            768..=769 => 0x42,
            770..=771 => 0x41,
            772 => 0x41,
            773..=774 => 0x46,
            _ => -1,
        }
    }

    pub fn experience_clientbound(self) -> i32 {
        match self.protocol {
            754 => 0x48,
            755..=758 => 0x51,
            759 => 0x51,
            760 => 0x54,
            761 => 0x52,
            762..=763 => 0x56,
            764 => 0x58,
            765 => 0x5A,
            766..=767 => 0x5C,
            768..=769 => 0x61,
            770..=771 => 0x60,
            772 => 0x60,
            773..=774 => 0x65,
            _ => -1,
        }
    }

    pub fn abilities_clientbound(self) -> i32 {
        match self.protocol {
            754 => 0x30,
            755..=756 => 0x32,
            757..=758 => 0x32,
            759 => 0x2F,
            760 => 0x31,
            761 => 0x30,
            762..=763 => 0x34,
            764..=765 => 0x36,
            766..=767 => 0x38,
            768..=769 => 0x3A,
            770..=772 => 0x39,
            773..=774 => 0x3E,
            _ => -1,
        }
    }

    pub fn chat_clientbound(self) -> i32 {
        match self.protocol {
            754 => 0x0E,
            755..=758 => 0x0F,
            759 => 0x5F,
            760 => 0x62,
            761 => 0x60,
            762..=763 => 0x64,
            764 => 0x67,
            765 => 0x69,
            766..=767 => 0x6C,
            768..=769 => 0x73,
            770..=772 => 0x72,
            773..=774 => 0x77,
            _ => -1,
        }
    }

    pub fn chat_serverbound(self) -> i32 {
        match self.protocol {
            754..=758 => 0x03,
            759 => 0x04,
            760 => 0x05,
            761..=765 => 0x05,
            766..=767 => 0x06,
            768..=770 => 0x07,
            771..=774 => 0x08,
            _ => -1,
        }
    }

    pub fn move_full_serverbound(self) -> i32 {
        match self.protocol {
            754 => 0x13,
            755..=758 => 0x12,
            759 => 0x14,
            760 => 0x15,
            761 => 0x14,
            762 => 0x15,
            763 => 0x15,
            764 => 0x17,
            765 => 0x18,
            766..=767 => 0x1B,
            768..=769 => 0x1D,
            770 => 0x1D,
            771..=774 => 0x1E,
            _ => -1,
        }
    }

    pub fn move_position_serverbound(self) -> i32 {
        match self.protocol {
            754 => 0x12,
            755..=758 => 0x11,
            759 => 0x13,
            760 => 0x14,
            761 => 0x13,
            762 => 0x14,
            763 => 0x14,
            764 => 0x16,
            765 => 0x17,
            766..=767 => 0x1A,
            768..=769 => 0x1C,
            770 => 0x1C,
            771..=774 => 0x1D,
            _ => -1,
        }
    }

    pub fn move_rotation_serverbound(self) -> i32 {
        match self.protocol {
            754 => 0x14,
            755..=758 => 0x13,
            759 => 0x15,
            760 => 0x16,
            761 => 0x15,
            762..=763 => 0x16,
            764 => 0x18,
            765 => 0x19,
            766..=767 => 0x1C,
            768..=769 => 0x1E,
            770 => 0x1E,
            771..=774 => 0x1F,
            _ => -1,
        }
    }

    pub fn move_ground_serverbound(self) -> i32 {
        match self.protocol {
            754 => 0x15,
            755..=758 => 0x14,
            759 => 0x16,
            760 => 0x17,
            761 => 0x16,
            762..=763 => 0x17,
            764 => 0x19,
            765 => 0x1A,
            766..=767 => 0x1D,
            768..=769 => 0x1F,
            770 => 0x1F,
            771..=774 => 0x20,
            _ => -1,
        }
    }

    pub fn teleport_confirm_serverbound(self) -> i32 {
        if self.protocol >= 107 {
            0x00
        } else {
            -1
        }
    }

    pub fn is_movement_serverbound(self, packet_id: i32) -> bool {
        packet_id == self.move_full_serverbound()
            || packet_id == self.move_position_serverbound()
            || packet_id == self.move_rotation_serverbound()
            || packet_id == self.move_ground_serverbound()
    }

    pub fn keep_alive_serverbound(self) -> i32 {
        match self.protocol {
            754 => 0x10,
            755..=758 => 0x0F,
            759 => 0x11,
            760 => 0x12,
            761 => 0x11,
            762..=763 => 0x12,
            764 => 0x14,
            765 => 0x15,
            766..=767 => 0x18,
            768..=770 => 0x1A,
            771..=774 => 0x1B,
            _ => -1,
        }
    }

    pub fn client_settings_serverbound(self) -> i32 {
        match self.protocol {
            754 => 0x05,
            755..=758 => 0x05,
            759 => 0x07,
            760 => 0x08,
            761 => 0x07,
            762..=763 => 0x08,
            764 => 0x09,
            765 => 0x09,
            766..=767 => 0x0A,
            768..=770 => 0x0C,
            771..=774 => 0x0D,
            _ => -1,
        }
    }

    pub fn plugin_message_serverbound(self) -> i32 {
        match self.protocol {
            754 => 0x0B,
            755..=758 => 0x0A,
            759 => 0x0C,
            760 => 0x0D,
            761 => 0x0C,
            762 => 0x0D,
            763 => 0x0D,
            764 => 0x0F,
            765 => 0x10,
            766..=767 => 0x12,
            768..=770 => 0x14,
            771..=774 => 0x15,
            _ => -1,
        }
    }

    pub fn player_chat_session_serverbound(self) -> i32 {
        match self.protocol {
            754..=760 => -1,
            761 => 0x20,
            762..=765 => 0x06,
            766..=767 => 0x07,
            768..=770 => 0x08,
            771..=774 => 0x09,
            _ => -1,
        }
    }

    pub fn player_loaded_serverbound(self) -> i32 {
        match self.protocol {
            769..=770 => 0x2A,
            771..=774 => 0x2B,
            _ => -1,
        }
    }

    pub fn read_position(self, payload: &mut BytesMut) -> io::Result<Position> {
        let mut teleport_id = if self.is_modern_position() {
            VarInt::read(payload)?
        } else {
            0
        };
        if payload.len() < 24 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "short player position",
            ));
        }
        let x = payload.get_f64();
        let y = payload.get_f64();
        let z = payload.get_f64();
        if self.is_modern_position() {
            if payload.len() < 24 + 8 + 4 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "short modern player position",
                ));
            }
            payload.advance(24); 
        }
        let yaw = payload.get_f32();
        let pitch = payload.get_f32();
        if self.is_modern_position() {
            if payload.len() < 4 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "missing relative flags",
                ));
            }
            payload.advance(4);
        } else {
            if !payload.has_remaining() {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "missing relative flags",
                ));
            }
            payload.advance(1);
            if self.protocol >= 107 {
                teleport_id = VarInt::read(payload)?;
            }
            if (755..=761).contains(&self.protocol) && payload.has_remaining() {
                payload.advance(1);
            }
        }
        Ok(Position {
            x,
            y,
            z,
            yaw,
            pitch,
            teleport_id,
        })
    }

    pub fn read_chat(self, payload: &mut BytesMut) -> io::Result<String> {
        read_string(payload, 256)
    }

    pub fn read_movement(
        self,
        packet_id: i32,
        payload: &mut BytesMut,
    ) -> io::Result<Option<Movement>> {
        let is_full = packet_id == self.move_full_serverbound();
        let is_position = packet_id == self.move_position_serverbound();
        if !is_full && !is_position {
            return Ok(None);
        }
        if payload.len() < 24 + if is_full { 8 } else { 0 } + 1 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "short player movement",
            ));
        }
        let x = payload.get_f64();
        let y = payload.get_f64();
        let z = payload.get_f64();
        let (yaw, pitch) = if is_full {
            (payload.get_f32(), payload.get_f32())
        } else {
            (0.0, 0.0)
        };
        let flags = payload.get_u8();
        let on_ground = if self.protocol >= 768 {
            flags & 1 != 0
        } else {
            flags != 0
        };
        Ok(Some(Movement {
            position: Position {
                x,
                y,
                z,
                yaw,
                pitch,
                teleport_id: 0,
            },
            on_ground,
        }))
    }
}

pub fn read_string(payload: &mut BytesMut, max_chars: usize) -> io::Result<String> {
    let length = VarInt::read(payload)?;
    if length < 0 || length as usize > max_chars * 4 || length as usize > payload.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid protocol string length",
        ));
    }
    let bytes = payload.split_to(length as usize);
    let text = String::from_utf8(bytes.to_vec())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid protocol string"))?;
    if text.chars().count() > max_chars {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "protocol string exceeds limit",
        ));
    }
    Ok(text)
}
