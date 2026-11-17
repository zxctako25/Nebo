#[derive(Debug, Clone, Copy)]
pub(crate) struct Limits {
    pub max_players: i32,
    pub show_online_count: bool,
    pub compression_threshold: i32,
    pub connection_timeout_millis: u64,
    pub read_timeout_seconds: u64,
    pub max_connections_per_ip: i32,
    pub max_total_connections: i32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_players: 100,
            show_online_count: true,
            compression_threshold: 256,
            connection_timeout_millis: 10_000,
            read_timeout_seconds: 30,
            max_connections_per_ip: 0,
            max_total_connections: 1000,
        }
    }
}
