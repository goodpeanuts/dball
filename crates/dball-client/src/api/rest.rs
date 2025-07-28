use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use toml::Value;

pub mod mxnzp;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RestConfig {
    pub api_name: String,
    pub base_url: String,
    pub timeout_ms: Option<usize>,
    pub max_retries: Option<usize>,
    #[serde(default)]
    pub meta: HashMap<String, Value>,
}
