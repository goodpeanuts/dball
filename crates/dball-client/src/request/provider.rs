use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use serde::Deserialize;
use tokio::sync::{Mutex, Semaphore};

use super::ApiCommon;

/// Enum representing different API service providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApiProvider {
    /// MXNZP API provider
    Mxnzp,
}

impl ApiProvider {
    /// Get the QPS limit for this provider
    pub fn qps_limit(&self) -> u32 {
        match self {
            Self::Mxnzp => 1,
        }
    }

    /// Get the unique identifier for this provider
    pub fn id(&self) -> &'static str {
        match self {
            Self::Mxnzp => "mxnzp",
        }
    }
}

/// Trait representing a service provider with QPS limitations
pub trait Provider: Send + Sync + 'static {
    /// Unique identifier for the provider
    fn provider_id(&self) -> ApiProvider;

    /// Get the executor for this provider
    fn get_executor(&self) -> Arc<QpsLimitedExecutor> {
        let provider = self.provider_id();
        Arc::new(QpsLimitedExecutor::new(provider))
    }
}

/// Request that can be executed through a provider
#[expect(async_fn_in_trait)]
pub trait ProviderRequest: Send + 'static
where
    for<'de> Self::Response: Deserialize<'de>,
{
    type Response: ProviderResponse;

    /// Execute the actual HTTP request
    async fn execute(self, common: &ApiCommon) -> anyhow::Result<Self::Response>;
}

// #[expect(async_fn_in_trait)]
pub trait ProviderResponse: Send + 'static
where
    for<'de> Self: Deserialize<'de>,
{
    type Data: for<'de> Deserialize<'de>;

    fn get_code(&self) -> i32;
    fn get_msg(&self) -> String;
    fn get_data(&self) -> Option<&Self::Data>;
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
            semaphore: Arc::new(Semaphore::new(qps_limit as usize)),
            last_request_time: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Execute a request with QPS limiting
    pub async fn execute<R>(&self, request: R, common: &ApiCommon) -> anyhow::Result<R::Response>
    where
        R: ProviderRequest,
    {
        // Acquire semaphore permit
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to acquire semaphore permit: {}", e))?;

        // Calculate delay needed to respect QPS limit
        let delay = self.calculate_delay().await;
        if delay > Duration::ZERO {
            log::debug!(
                "Provider {} QPS limiting: waiting {:?}",
                self.provider.id(),
                delay
            );
            tokio::time::sleep(delay).await;
        }

        // Update last request time
        {
            let mut last_time = self.last_request_time.lock().await;
            *last_time = Instant::now();
        }

        log::debug!("Executing request for provider: {}", self.provider.id());

        // Execute the actual request
        request.execute(common).await
    }

    async fn calculate_delay(&self) -> Duration {
        let last_time = self.last_request_time.lock().await;
        let elapsed = last_time.elapsed();
        let qps = self.provider.qps_limit();
        if qps == 0 {
            log::warn!(
                "QPS limit for provider {} is 0, skipping delay calculation to avoid division by zero.",
                self.provider.id()
            );
            return Duration::ZERO;
        }
        let min_interval = Duration::from_secs_f64(1.0 / qps as f64);

        if elapsed < min_interval {
            min_interval - elapsed
        } else {
            Duration::ZERO
        }
    }
}

/// Macro to implement Provider for a type
#[macro_export]
macro_rules! impl_provider {
    ($type:ty, $provider:expr) => {
        impl $crate::request::provider::Provider for $type {
            fn provider_id(&self) -> $crate::request::provider::ApiProvider {
                $provider
            }
        }
    };
}

pub use impl_provider;
