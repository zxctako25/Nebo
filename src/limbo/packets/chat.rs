use bytes::{BufMut, BytesMut};
use serde_json::json;

use crate::protocol::io::{ProtocolUtils, VarInt};

pub fn chat_packet(protocol: i32, packet_id: i32, text: &str) -> anyhow::Result<BytesMut> {
    let mut body = BytesMut::with_capacity(text.len() + 16);
    VarInt::write(&mut body, packet_id);
    let component = serde_json::to_string(&json!({"text": text}))?;
    ProtocolUtils::write_string(&mut body, &component);
    if protocol < 759 {
        body.put_u8(1); 
        body.extend_from_slice(&[0u8; 16]); 
    } else if protocol == 759 {
        VarInt::write(&mut body, 1); 
    } else {
        body.put_u8(0); 
    }
    Ok(body)
}

pub fn action_bar_packet(protocol: i32, packet_id: i32, text: &str) -> anyhow::Result<BytesMut> {
    let mut body = chat_packet(protocol, packet_id, text)?;
    if protocol >= 759 {
        if let Some(is_action_bar) = body.last_mut() {
            *is_action_bar = 1;
        }
    }
    Ok(body)
}
