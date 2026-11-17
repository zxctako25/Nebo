use super::packet::ConnectionState;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct HandshakeData {
    pub protocol: i32,
    pub host: String,
    pub port: u16,
    pub next_state: ConnectionState,
}

#[derive(Debug, Clone)]
pub struct LoginStartData {
    pub name: String,
    pub uuid: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub struct LoginSuccessData {
    pub uuid: Uuid,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct PluginResponseData {
    pub message_id: i32,
    pub successful: bool,
    pub data: Option<Vec<u8>>,
}
