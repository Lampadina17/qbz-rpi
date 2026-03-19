#[derive(Debug, Clone)]
pub struct WsTransportConfig {
    pub endpoint_url: String,
    pub jwt_qws: Option<String>,
    pub reconnect_backoff_ms: u64,
    pub reconnect_backoff_max_ms: u64,
    pub connect_timeout_ms: u64,
    pub keepalive_interval_ms: u64,
    pub auto_subscribe: bool,
    pub subscribe_channels: Vec<Vec<u8>>,
    pub qcloud_proto: u32,
}

impl Default for WsTransportConfig {
    fn default() -> Self {
        Self {
            endpoint_url: String::new(),
            jwt_qws: None,
            reconnect_backoff_ms: 2_000,
            reconnect_backoff_max_ms: 30_000,
            connect_timeout_ms: 10_000,
            keepalive_interval_ms: 30_000,
            auto_subscribe: true,
            subscribe_channels: Vec::new(),
            qcloud_proto: 1,
        }
    }
}
