use std::{collections::HashMap, fs, path::Path, sync::LazyLock};

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator as _;
use toml::Value;

use crate::{
    ENV_GUARD,
    api::{Protocol, provider::ApiProvider},
};

const API_CONFIG_FILE: &str = "api.toml";
const API_DIR: &str = "api";

/// Get all valid protocol names for error reporting
fn get_valid_protocols() -> Vec<String> {
    Protocol::iter().map(|p| p.to_string()).collect()
}

/// Get all valid provider names for error reporting  
fn get_valid_providers() -> Vec<&'static str> {
    ApiProvider::iter().map(|p| p.id()).collect()
}

pub static API_CONFIG: LazyLock<Result<ApiConfig>> = LazyLock::new(|| match ENV_GUARD.as_ref() {
    Ok(env_file_path) => {
        let root_path = env_file_path
            .parent()
            .context("Could not get parent directory of .env file")?;

        // Use new multi-file loading approach
        ApiConfig::new(root_path.join(API_CONFIG_FILE), root_path.join(API_DIR))
    }
    Err(e) => {
        log::error!("Failed to load .env file: {e}, using default config");
        Err(anyhow::anyhow!("Failed to load .env file: {e}"))
    }
});

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
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

impl ProviderConfig {
    /// Merge another `ProviderConfig` into this one
    /// Existing entries in self take priority over entries in other
    pub fn merge_with(&mut self, other: Self) {
        #[expect(clippy::iter_over_hash_type)]
        for (key, value) in other.rest {
            self.rest.entry(key).or_insert(value);
        }

        #[expect(clippy::iter_over_hash_type)]
        for (key, value) in other.ws {
            self.ws.entry(key).or_insert(value);
        }

        #[expect(clippy::iter_over_hash_type)]
        for (key, value) in other.fix {
            self.fix.entry(key).or_insert(value);
        }
        #[expect(clippy::iter_over_hash_type)]
        for (key, value) in other.mq {
            self.mq.entry(key).or_insert(value);
        }

        #[expect(clippy::iter_over_hash_type)]
        for (key, value) in other.grpc {
            self.grpc.entry(key).or_insert(value);
        }
    }
}

impl ApiConfig {
    /// Create `ApiConfig` with multi-file support
    /// Loads main config from `api.toml` and additional configs from `api/`
    pub fn new<P: AsRef<Path>>(api_toml: P, api_dir: P) -> Result<Self> {
        // Try to load main config file (optional)
        let mut config = match Self::new_api_toml(&api_toml) {
            Ok(config) => {
                log::debug!("Loaded main config from: {}", api_toml.as_ref().display());
                config
            }
            Err(e) => {
                log::debug!(
                    "Main config file not found or invalid, starting with empty config: {e}",
                );
                Self::default()
            }
        };

        // Load additional provider configs from api/ directory
        let provider_configs = Self::load_provider_configs_from_dir(&api_dir)?;

        #[expect(clippy::iter_over_hash_type)]
        for (api_provider, provider_config) in provider_configs {
            config.merge_provider_config(api_provider, provider_config);
        }

        // Validate that at least one provider is configured
        if config.is_empty() {
            return Err(anyhow::anyhow!(
                "No valid configuration found. Please ensure either {} exists or provide configs in {}",
                api_toml.as_ref().display(),
                api_dir.as_ref().display()
            ));
        }

        Ok(config)
    }

    pub fn new_api_toml<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config_path_str = config_path.as_ref().to_string_lossy().to_string();
        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {config_path_str}"))?;

        // Parse as TOML Value first to validate structure
        let toml_value: toml::Value = toml::from_str(&content).with_context(|| {
            format!("Failed to parse TOML content in main config file: {config_path_str}")
        })?;

        // Validate that top-level keys are valid provider names
        Self::validate_providers(&toml_value, &config_path)?;

        // Now parse as ApiConfig
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML config: {config_path_str}"))?;

        Ok(config)
    }

    /// Validate the structure of the main config file (api.toml)
    /// Ensures all top-level keys are valid provider names
    fn validate_providers<P: AsRef<Path>>(toml_value: &toml::Value, config_path: P) -> Result<()> {
        let table = toml_value.as_table().ok_or_else(|| {
            anyhow::anyhow!(
                "Main config file {} must contain a TOML table at root level",
                config_path.as_ref().display()
            )
        })?;

        let invalid_keys: Vec<&String> = table
            .keys()
            .filter(|key| key.parse::<ApiProvider>().is_err())
            .collect();

        if !invalid_keys.is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid provider names found in {}: {invalid_keys:?}.\nValid providers are: {:?}",
                config_path.as_ref().display(),
                get_valid_providers()
            ));
        }

        Ok(())
    }

    /// Load provider configurations from `api/<provider_name>.toml` files
    fn load_provider_configs_from_dir<P: AsRef<Path>>(
        api_dir: P,
    ) -> Result<HashMap<ApiProvider, ProviderConfig>> {
        let mut provider_configs = HashMap::new();

        if !api_dir.as_ref().exists() {
            log::debug!("API directory not found: {}", api_dir.as_ref().display());
            return Ok(provider_configs);
        }

        let entries = fs::read_dir(&api_dir).with_context(|| {
            format!(
                "Failed to read API directory: {}",
                api_dir.as_ref().display()
            )
        })?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // ignore unknown provider

                    let Ok(api_provider) = file_stem.parse::<ApiProvider>() else {
                        log::warn!(
                            "Unknown provider name '{file_stem}' found in config file, ignoring",
                        );
                        continue;
                    };

                    match Self::load_provider_config_in_dir(&path) {
                        Ok(config) => {
                            provider_configs.insert(api_provider, config);
                            log::debug!("Successfully loaded config for provider: {file_stem}");
                        }
                        Err(e) => {
                            log::warn!("Failed to load config for provider {file_stem}: {e}");
                        }
                    }
                }
            }
        }

        Ok(provider_configs)
    }

    /// Load a single provider config file
    fn load_provider_config_in_dir<P: AsRef<Path>>(config_path: P) -> Result<ProviderConfig> {
        let content = fs::read_to_string(&config_path).with_context(|| {
            format!(
                "Failed to read provider config file: {}",
                config_path.as_ref().display()
            )
        })?;

        // Parse as TOML Value first to validate structure
        let toml_value: toml::Value = toml::from_str(&content).with_context(|| {
            format!(
                "Failed to parse TOML content in: {}",
                config_path.as_ref().display()
            )
        })?;

        // Validate that top-level keys are valid protocols
        Self::validate_protocol_folder_toml(&toml_value, &config_path)?;

        // Now parse as ProviderConfig (simplified format)
        let config: ProviderConfig = toml::from_str(&content).with_context(|| {
            format!(
                "Failed to parse provider TOML config: {}",
                config_path.as_ref().display()
            )
        })?;

        Ok(config)
    }

    /// Validate the structure of a provider config file (api/<provider>.toml)
    /// Ensures all top-level keys are valid protocol names
    fn validate_protocol_folder_toml<P: AsRef<Path>>(
        toml_value: &toml::Value,
        config_path: P,
    ) -> Result<()> {
        let table = toml_value.as_table().ok_or_else(|| {
            anyhow::anyhow!(
                "Provider config file {} must contain a TOML table at root level",
                config_path.as_ref().display()
            )
        })?;

        let invalid_keys: Vec<&String> = table
            .keys()
            .filter(|key| key.parse::<Protocol>().is_err())
            .collect();

        if !invalid_keys.is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid protocol names found in {}: {invalid_keys:?}. Valid protocols are: {:?}",
                config_path.as_ref().display(),
                get_valid_protocols()
            ));
        }

        Ok(())
    }

    /// Merge a provider config into the main config
    fn merge_provider_config(&mut self, provider: ApiProvider, provider_config: ProviderConfig) {
        match provider {
            ApiProvider::Mxnzp => {
                if let Some(existing) = &mut self.mxnzp {
                    existing.merge_with(provider_config);
                } else {
                    self.mxnzp = Some(provider_config);
                }
            }
            ApiProvider::Binance => {
                if let Some(existing) = &mut self.binance {
                    existing.merge_with(provider_config);
                } else {
                    self.binance = Some(provider_config);
                }
            }
            ApiProvider::Custom => {
                if let Some(existing) = &mut self.custom {
                    existing.merge_with(provider_config);
                } else {
                    self.custom = Some(provider_config);
                }
            }
        }
    }

    /// Check if the config is empty (no providers configured)
    fn is_empty(&self) -> bool {
        self.mxnzp.is_none() && self.binance.is_none() && self.custom.is_none()
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
            ApiProvider::Binance => self
                .binance
                .as_ref()
                .with_context(|| "Binance provider config not found"),
            ApiProvider::Custom => self
                .custom
                .as_ref()
                .with_context(|| "Custom provider config not found"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplified_provider_config_conversion() {
        let mut simplified = ProviderConfig::default();
        simplified.rest.insert(
            "test_api".to_owned(),
            super::super::rest::RestConfig {
                api_name: "test".to_owned(),
                base_url: "https://example.com".to_owned(),
                timeout_ms: Some(5000),
                max_retries: None,
                meta: HashMap::new(),
            },
        );

        let provider_config = simplified;
        assert!(provider_config.rest.contains_key("test_api"));
        assert_eq!(provider_config.rest["test_api"].api_name, "test");
    }

    #[test]
    fn test_provider_config_merge() {
        let mut config1 = ProviderConfig::default();
        config1.rest.insert(
            "api1".to_owned(),
            super::super::rest::RestConfig {
                api_name: "first".to_owned(),
                base_url: "https://first.com".to_owned(),
                timeout_ms: Some(1000),
                max_retries: None,
                meta: HashMap::new(),
            },
        );

        let mut config2 = ProviderConfig::default();
        config2.rest.insert(
            "api2".to_owned(),
            super::super::rest::RestConfig {
                api_name: "second".to_owned(),
                base_url: "https://second.com".to_owned(),
                timeout_ms: Some(2000),
                max_retries: None,
                meta: HashMap::new(),
            },
        );
        config2.rest.insert(
            "api1".to_owned(), // Duplicate key - should not override config1
            super::super::rest::RestConfig {
                api_name: "should_not_override".to_owned(),
                base_url: "https://should-not-override.com".to_owned(),
                timeout_ms: Some(9999),
                max_retries: None,
                meta: HashMap::new(),
            },
        );

        config1.merge_with(config2);

        assert_eq!(config1.rest.len(), 2);
        assert_eq!(config1.rest["api1"].api_name, "first"); // Should not be overridden
        assert_eq!(config1.rest["api2"].api_name, "second"); // Should be added
    }

    #[test]
    fn test_api_config_empty_check() {
        let empty_config = ApiConfig::default();
        assert!(empty_config.is_empty());

        let non_empty_config = ApiConfig {
            mxnzp: Some(ProviderConfig::default()),
            ..Default::default()
        };
        assert!(!non_empty_config.is_empty());
    }

    #[tokio::test]
    async fn test_load_provider_configs_empty_directory() {
        let temp_dir = std::env::temp_dir().join("dball_test_empty");
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

        let result = ApiConfig::load_provider_configs_from_dir(&temp_dir);
        assert!(result.is_ok());
        assert!(result.expect("Failed to load provider configs").is_empty());

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_multi_file_config_loading() {
        use std::io::Write as _;

        let temp_dir = std::env::temp_dir().join("dball_test_multi");
        let api_dir = temp_dir.join("api");
        std::fs::create_dir_all(&api_dir).expect("Failed to create temp directory");

        // Create main config file
        let main_config_content = r#"
[mxnzp.rest.main_api]
api_name = "main_api"
base_url = "https://main.example.com"
timeout_ms = 5000
"#;
        let main_config_path = temp_dir.join(API_CONFIG_FILE);
        let mut main_file =
            std::fs::File::create(&main_config_path).expect("Failed to create main config file");
        main_file
            .write_all(main_config_content.as_bytes())
            .expect("Failed to write main config");

        // Create provider-specific config file
        let provider_config_content = r#"
[rest.provider_api]
api_name = "provider_api"
base_url = "https://provider.example.com"
timeout_ms = 3000

[rest.main_api]
api_name = "should_not_override"
base_url = "https://should-not-override.example.com"
timeout_ms = 9999
"#;
        let provider_config_path = api_dir.join("mxnzp.toml");
        let mut provider_file = std::fs::File::create(&provider_config_path)
            .expect("Failed to create provider config file");
        provider_file
            .write_all(provider_config_content.as_bytes())
            .expect("Failed to write provider config");

        // Test loading
        let result = ApiConfig::new(&main_config_path, &api_dir);
        assert!(result.is_ok());

        let config = result.expect("Failed to load API config");
        assert!(config.mxnzp.is_some());

        let mxnzp_config = config.mxnzp.expect("Failed to load mxnzp config");
        assert_eq!(mxnzp_config.rest.len(), 2);

        // Main config should take priority
        assert_eq!(mxnzp_config.rest["main_api"].api_name, "main_api");
        assert_eq!(
            mxnzp_config.rest["main_api"].base_url,
            "https://main.example.com"
        );
        assert_eq!(mxnzp_config.rest["main_api"].timeout_ms, Some(5000));

        // Provider-specific config should be added
        assert_eq!(mxnzp_config.rest["provider_api"].api_name, "provider_api");
        assert_eq!(
            mxnzp_config.rest["provider_api"].base_url,
            "https://provider.example.com"
        );

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_main_config_validation() {
        use std::io::Write as _;

        let temp_dir = std::env::temp_dir().join("dball_test_main_validation");
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        // Test valid main config
        let valid_config = r#"
[mxnzp.rest.test_api]
api_name = "test"
base_url = "https://example.com"
"#;
        let valid_path = temp_dir.join("valid.toml");
        let mut file =
            std::fs::File::create(&valid_path).expect("Failed to create valid config file");
        file.write_all(valid_config.as_bytes())
            .expect("Failed to write valid config");

        let result = ApiConfig::new_api_toml(&valid_path);
        assert!(result.is_ok(), "Valid config should parse successfully");

        // Test invalid main config
        let invalid_config = r#"
[invalid_provider.rest.test_api]
api_name = "test"
base_url = "https://example.com"
"#;
        let invalid_path = temp_dir.join("invalid.toml");
        let mut file =
            std::fs::File::create(&invalid_path).expect("Failed to create invalid config file");
        file.write_all(invalid_config.as_bytes())
            .expect("Failed to write invalid config");

        let result = ApiConfig::new_api_toml(&invalid_path);
        assert!(result.is_err(), "Invalid provider name should cause error");

        let error_msg = result
            .expect_err("Invalid provider name should cause error")
            .to_string();
        assert!(
            error_msg.contains("Invalid provider names") && error_msg.contains("invalid_provider"),
            "Error should mention invalid provider name"
        );

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_provider_config_validation() {
        use std::io::Write as _;

        let temp_dir = std::env::temp_dir().join("dball_test_provider_validation");
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        // Test valid provider config
        let valid_config = r#"
[rest.test_api]
api_name = "test"
base_url = "https://example.com"

[ws.test_ws]
api_name = "test_ws"
base_url = "wss://example.com"
"#;
        let valid_path = temp_dir.join("valid_provider.toml");
        let mut file =
            std::fs::File::create(&valid_path).expect("Failed to create valid config file");
        file.write_all(valid_config.as_bytes())
            .expect("Failed to write valid config");

        let result = ApiConfig::load_provider_config_in_dir(&valid_path).map_err(|e| {
            log::error!("Failed to load provider config: {e}");
            e
        });
        assert!(
            result.is_ok(),
            "Valid provider config should parse successfully"
        );

        // Test invalid provider config
        let invalid_config = r#"
[invalid_protocol.test_api]
api_name = "test"
base_url = "https://example.com"

[rest.test_rest]
api_name = "test_rest"
base_url = "https://example.com"
"#;
        let invalid_path = temp_dir.join("invalid_provider.toml");
        let mut file =
            std::fs::File::create(&invalid_path).expect("Failed to create invalid config file");
        file.write_all(invalid_config.as_bytes())
            .expect("Failed to write invalid config");

        let result = ApiConfig::load_provider_config_in_dir(&invalid_path);
        assert!(result.is_err(), "Invalid protocol name should cause error");

        let error_msg = result
            .expect_err("Invalid protocol name should cause error")
            .to_string();
        assert!(
            error_msg.contains("Invalid protocol names") && error_msg.contains("invalid_protocol"),
            "Error should mention invalid protocol name"
        );

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_config_validation_integration() {
        // Test that ApiConfig::new rejects invalid provider names
        let temp_dir = std::env::temp_dir().join("dball_test_invalid_integration");
        std::fs::create_dir_all(&temp_dir).expect("Failed to create temp dir");

        let invalid_main_config = r#"
[invalid_provider.rest.test_api]
api_name = "test"
base_url = "https://example.com"

[another_invalid.ws.test_ws]
api_name = "test_ws"
base_url = "wss://example.com"
"#;
        let invalid_path = temp_dir.join("api_invalid.toml");
        std::fs::write(&invalid_path, invalid_main_config).expect("Failed to write invalid config");

        let result = ApiConfig::new_api_toml(&invalid_path);
        assert!(result.is_err(), "Should reject invalid provider names");

        let error = result.expect_err("Should reject invalid provider names");
        let error_msg = error.to_string();
        log::debug!("Error message: {error_msg}");

        assert!(error_msg.contains("Invalid provider names"));
        assert!(error_msg.contains("invalid_provider"));
        assert!(error_msg.contains("another_invalid"));

        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
