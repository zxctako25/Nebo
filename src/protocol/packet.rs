#[derive(Debug, Clone, Copy)]
pub enum HandshakePacket {
    Serverbound,
}

#[derive(Debug, Clone, Copy)]
pub enum StatusPacket {
    RequestServerbound,
    PingServerbound,
    ResponseClientbound,
    PongClientbound,
}

#[derive(Debug, Clone, Copy)]
pub enum LoginPacket {
    StartServerbound,
    EncryptionResponseServerbound,
    PluginResponseServerbound,
    LoginAcknowledgedServerbound,

    DisconnectClientbound,
    EncryptionRequestClientbound,
    SuccessClientbound,
    SetCompressionClientbound,
    PluginRequestClientbound,
    CookieRequestClientbound,
}

#[derive(Debug, Clone, Copy)]
pub enum ConfigurationPacket {
    ClientInformationServerbound,
    PluginMessageServerbound,
    FinishConfigurationServerbound,
    KeepAliveServerbound,
    PongServerbound,
    ResourcePackResponseServerbound,

    PluginMessageClientbound,
    DisconnectClientbound,
    FinishConfigurationClientbound,
    KeepAliveClientbound,
    PingClientbound,
    RegistryDataClientbound,
    ResourcePackClientbound,
    UpdateTagsClientbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    AwaitingLoginAck,
    Configuration,
    Play,
}

impl ConnectionState {
    pub fn id(&self) -> i32 {
        match self {
            ConnectionState::Handshake => 0,
            ConnectionState::Status => 1,
            ConnectionState::Login => 2,
            ConnectionState::AwaitingLoginAck => 3,
            ConnectionState::Configuration => 4,
            ConnectionState::Play => 5,
        }
    }

    pub fn by_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(ConnectionState::Handshake),
            1 => Some(ConnectionState::Status),
            2 => Some(ConnectionState::Login),
            _ => None,
        }
    }
}
