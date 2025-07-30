use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use strum_macros::{Display, EnumIter};
use tokio::sync::{Mutex, Semaphore};

use crate::api::{ApiCommon, Protocol};

pub mod mxnzp;

/// Enum representing different API service providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumIter)]
pub enum ApiProvider {
    /// MXNZP API provider
    #[strum(to_string = "mxnzp")]
    Mxnzp,
    /// Binance API provider
    #[strum(to_string = "binance")]
    Binance,
    /// Custom API provider
    #[strum(to_string = "custom")]
    Custom,
}

impl ApiProvider {
    /// Get the QPS limit for this provider
    pub fn qps_limit(&self) -> usize {
        match self {
            Self::Mxnzp => 1,
            Self::Binance => 10,
            Self::Custom => 5,
        }
    }

    /// Get the unique identifier for this provider
    pub fn id(&self) -> &'static str {
        match self {
            Self::Mxnzp => "mxnzp",
            Self::Binance => "binance",
            Self::Custom => "custom",
        }
    }

    /// Get namespace for this provider (`provider.protocol.api_name`)
    #[expect(unused)]
    pub fn config_namespace(&self, protocol: crate::api::Protocol, api_name: &str) -> String {
        format!(
            "{}.{}.{}",
            self.id(),
            protocol.to_string().to_lowercase(),
            api_name
        )
    }

    /// Get the environment variable prefix for authentication
    /// (e.g., `API_MXNZP_APP_ID`)
    #[expect(unused)]
    pub fn auth_env_prefix(&self) -> String {
        format!("API_{}", self.id().to_uppercase())
    }
}

/// Request that can be executed through a provider (protocol-agnostic)
#[expect(async_fn_in_trait)]
pub trait ProviderRequest: Send + 'static {
    type Response: ProviderResponse;

    /// Execute the actual request (protocol-agnostic)
    async fn execute(self) -> anyhow::Result<Self::Response>;
}

/// Response from a provider request (protocol-agnostic)
pub trait ProviderResponse: Send + 'static {
    type Data;

    fn get_code(&self) -> i32;
    fn get_msg(&self) -> String;
    fn get_data(&self) -> Option<&Self::Data>;
}

/// Provider trait with embedded QPS-limited executor
pub trait Provider: Send + Sync + 'static {
    /// Get the provider type
    fn provider_type(&self) -> ApiProvider;

    /// Get the embedded executor for this provider
    fn executor(&self) -> &QpsLimitedExecutor;

    /// Create `ApiCommon` for a specific API
    fn create_api_common(&self, protocol: Protocol, api_name: &str) -> anyhow::Result<ApiCommon> {
        let provider_type = self.provider_type();

        let api = crate::api::config::API_CONFIG
            .as_ref()
            .map_err(|e| anyhow::anyhow!("Failed to load API config: {}", e))?
            .get_api_config(provider_type, protocol, api_name)?;

        let common = ApiCommon {
            name: api.api_name().to_owned(),
            protocol,
            url: api.base_url().to_owned(),
            timeout_ms: api.timeout_ms(),
        };

        Ok(common)
    }

    /// Execute a request with QPS limiting
    async fn execute_request<R>(&self, request: R) -> anyhow::Result<R::Response>
    where
        R: ProviderRequest,
    {
        self.executor().execute(request).await
    }
}

/// QPS-limited executor that manages request queues and rate limiting
#[derive(Debug)]
pub struct QpsLimitedExecutor {
    provider: ApiProvider,
    semaphore: Arc<Semaphore>,
    last_request_time: Arc<Mutex<Instant>>,
}

impl QpsLimitedExecutor {
    pub fn new(provider: ApiProvider) -> Self {
        let qps_limit = provider.qps_limit();
        Self {
            provider,
            semaphore: Arc::new(Semaphore::new(qps_limit)),
            last_request_time: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Execute a request with QPS limiting
    pub async fn execute<R>(&self, request: R) -> anyhow::Result<R::Response>
    where
        R: ProviderRequest,
    {
        // Acquire semaphore permit to ensure serial execution
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to acquire semaphore permit: {}", e))?;

        // Calculate delay based on last request time
        let delay = {
            let last_time = self.last_request_time.lock().await;
            let elapsed = last_time.elapsed();
            let qps = self.provider.qps_limit();

            if qps == 0 {
                log::warn!(
                    "QPS limit for provider {} is 0, skipping delay calculation to avoid division by zero.",
                    self.provider.id()
                );
                Duration::ZERO
            } else {
                // Calculate minimum interval between requests (1 second / QPS)
                let min_interval = Duration::from_secs_f64(1.0 / qps as f64);

                if elapsed < min_interval {
                    min_interval - elapsed
                } else {
                    Duration::ZERO
                }
            }
        };

        // Apply delay if needed
        if delay > Duration::ZERO {
            log::debug!(
                "Provider {} QPS limiting: waiting {:?}",
                self.provider.id(),
                delay
            );
            tokio::time::sleep(delay).await;
        }

        let response = request.execute().await;

        // Update last request time to now (right before execution)
        {
            let mut last_time = self.last_request_time.lock().await;
            *last_time = Instant::now();
        }

        log::debug!("Executing request for provider: {}", self.provider.id());
        response
    }
}

impl std::str::FromStr for ApiProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mxnzp" => Ok(Self::Mxnzp),
            "binance" => Ok(Self::Binance),
            "custom" => Ok(Self::Custom),
            _ => Err(format!("Invalid provider: {s}")),
        }
    }
}
