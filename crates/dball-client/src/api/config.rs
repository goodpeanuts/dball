use std::{collections::HashMap, fs, path::Path, sync::LazyLock};

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use toml::Value;

use crate::{
    ENV_GUARD,
    api::{Protocol, provider::ApiProvider},
};

pub static API_CONFIG: LazyLock<Result<ApiConfig>> = LazyLock::new(|| match ENV_GUARD.as_ref() {
    Ok(env_file_path) => {
        let root_path = env_file_path
            .parent()
            .context("Could not get parent directory of .env file")?;

        ApiConfig::new(root_path.join("api.toml"))
    }
    Err(e) => {
        log::error!("Failed to load .env file: {e}, using default config");
        Err(anyhow::anyhow!("Failed to load .env file: {e}"))
    }
});

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiConfig {
    #[serde(default)]
    pub mxnzp: Option<ProviderConfig>,
    #[serde(default)]
    pub binance: Option<ProviderConfig>,
    #[serde(default)]
    pub custom: Option<ProviderConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ProviderConfig {
    #[serde(default)]
    pub rest: HashMap<String, super::rest::RestConfig>,
    #[serde(default)]
    pub ws: HashMap<String, super::websocket::WebSocketConfig>,
    #[serde(default)]
    pub fix: HashMap<String, super::fix::FixConfig>,
    #[serde(default)]
    pub mq: HashMap<String, MqConfig>,
    #[serde(default)]
    pub grpc: HashMap<String, GrpcConfig>,
}

impl ApiConfig {
    pub fn new<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config_path_str = config_path.as_ref().to_string_lossy().to_string();
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {config_path_str}"))?;

        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML config: {config_path_str}"))?;

        Ok(config)
    }

    /// Get API configuration for a specific provider and protocol
    pub fn get_api_config(
        &self,
        provider: ApiProvider,
        protocol: Protocol,
        api_name: &str,
    ) -> Result<ApiConfigEntry> {
        let provider_config = self.get_provider_config(provider)?;

        match protocol {
            Protocol::Rest => {
                let rest_config = provider_config.rest.get(api_name).with_context(|| {
                    format!(
                        "REST API '{api_name}' not found for provider '{}'",
                        provider.id()
                    )
                })?;
                Ok(ApiConfigEntry::Rest(rest_config.clone()))
            }
            Protocol::WebSocket => {
                let ws_config = provider_config.ws.get(api_name).with_context(|| {
                    format!(
                        "WebSocket API '{}' not found for provider '{}'",
                        api_name,
                        provider.id()
                    )
                })?;
                Ok(ApiConfigEntry::WebSocket(ws_config.clone()))
            }
            _ => Err(anyhow::anyhow!(
                "Protocol {:?} not yet implemented",
                protocol
            )),
        }
    }

    fn get_provider_config(&self, provider: ApiProvider) -> Result<&ProviderConfig> {
        match provider {
            ApiProvider::Mxnzp => self
                .mxnzp
                .as_ref()
                .with_context(|| "MXNZP provider config not found"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ApiConfigEntry {
    Rest(super::rest::RestConfig),
    WebSocket(super::websocket::WebSocketConfig),
    Fix(super::fix::FixConfig),
    Mq(MqConfig),
    Grpc(GrpcConfig),
}

impl ApiConfigEntry {
    pub fn api_name(&self) -> &str {
        match self {
            Self::Rest(config) => &config.api_name,
            Self::WebSocket(config) => &config.api_name,
            Self::Fix(config) => &config.api_name,
            Self::Mq(config) => &config.api_name,
            Self::Grpc(config) => &config.api_name,
        }
    }

    pub fn timeout_ms(&self) -> Option<usize> {
        match self {
            Self::Rest(config) => config.timeout_ms,
            Self::WebSocket(config) => config.timeout_ms,
            Self::Fix(config) => config.timeout_ms,
            Self::Mq(config) => config.timeout_ms,
            Self::Grpc(config) => config.timeout_ms,
        }
    }

    pub fn meta(&self) -> &HashMap<String, Value> {
        match self {
            Self::Rest(config) => &config.meta,
            Self::WebSocket(config) => &config.meta,
            Self::Fix(config) => &config.meta,
            Self::Mq(config) => &config.meta,
            Self::Grpc(config) => &config.meta,
        }
    }

    pub fn base_url(&self) -> &str {
        match self {
            Self::Rest(config) => &config.base_url,
            Self::WebSocket(config) => &config.base_url,
            Self::Fix(_) => "", // FIX 协议没有 URL
            Self::Mq(config) => &config.url,
            Self::Grpc(config) => &config.base_url,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqConfig {
    pub api_name: String,
    pub url: String,
    pub exchange: String,
    pub routing_key: String,
    pub timeout_ms: Option<usize>,
    #[serde(default)]
    pub meta: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GrpcConfig {
    pub api_name: String,
    pub base_url: String,
    pub timeout_ms: Option<usize>,
    #[serde(default)]
    pub meta: HashMap<String, Value>,
}
