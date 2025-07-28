use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use toml::Value;

/// FIX 协议配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FixConfig {
    pub api_name: String,
    pub host: String,
    pub port: u16,
    pub version: String,
    pub timeout_ms: Option<usize>,
    #[serde(default)]
    pub meta: HashMap<String, Value>,
}
