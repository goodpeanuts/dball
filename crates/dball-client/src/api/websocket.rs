use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use toml::Value;

/// WebSocket 配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WebSocketConfig {
    pub api_name: String,
    pub base_url: String,
    pub timeout_ms: Option<usize>,
    pub heartbeat_interval: Option<usize>,
    #[serde(default)]
    pub meta: HashMap<String, Value>,
}
