use bytes::BytesMut;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use uuid::Uuid;

use crate::protocol::constants::{VELOCITY_MODERN_DEFAULT, VELOCITY_MODERN_MAX};
use crate::protocol::io::{ProtocolUtils, VarInt};

pub const CHANNEL: &str = "velocity:player_info";

pub fn create_forwarding_data(
    secret: &str,
    requested_version: i32,
    address: &str,
    uuid: &Uuid,
    name: &str,
) -> Option<Vec<u8>> {
    let mut version = requested_version.min(VELOCITY_MODERN_MAX);
    if version < VELOCITY_MODERN_DEFAULT {
        version = VELOCITY_MODERN_DEFAULT;
    }

    let mut forwarded = BytesMut::with_capacity(2048);
    VarInt::write(&mut forwarded, version);
    ProtocolUtils::write_string(&mut forwarded, address);
    ProtocolUtils::write_uuid(&mut forwarded, uuid);
    ProtocolUtils::write_string(&mut forwarded, name);
    VarInt::write(&mut forwarded, 0);

    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(&forwarded);
    let signature = mac.finalize().into_bytes();

    let mut result = Vec::with_capacity(signature.len() + forwarded.len());
    result.extend_from_slice(&signature);
    result.extend_from_slice(&forwarded);
    Some(result)
}
