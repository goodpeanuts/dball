use std::sync::LazyLock;

use strum_macros::Display;

use super::{Provider, QpsLimitedExecutor};
use crate::api::provider::ApiProvider;
use crate::parse_from_env;

/// Global MXNZP provider instance
pub static MXNZP_PROVIDER: LazyLock<MxnzpProvider> = LazyLock::new(|| MxnzpProvider {
    app_id: parse_from_env("MXNZP_APP_ID"),
    app_secret: parse_from_env("MXNZP_APP_SECRET"),
    executor: QpsLimitedExecutor::new(ApiProvider::Mxnzp),
});

pub const RETURN_CODE_SUCCESS: i32 = 1;

/// MXNZP API provider with embedded QPS executor
#[derive(Debug)]
pub struct MxnzpProvider {
    app_id: Option<String>,
    app_secret: Option<String>,
    executor: QpsLimitedExecutor,
}

#[derive(Display)]
pub enum MxnzpApi {
    #[strum(to_string = "get_latest_lottery")]
    GetLatestLottery,
    #[strum(to_string = "get_specified_lottery")]
    GetSpecifiedLottery,
}

impl MxnzpProvider {
    /// return the authentication configuration
    pub fn get_auth_config(&self) -> anyhow::Result<(String, String)> {
        if let (Some(app_id), Some(app_secret)) = (self.app_id.as_ref(), self.app_secret.as_ref()) {
            Ok((app_id.clone(), app_secret.clone()))
        } else {
            Err(anyhow::anyhow!(
                "Missing app_id or app_secret in MXNZP provider"
            ))
        }
    }
}

impl Provider for MxnzpProvider {
    fn provider_type(&self) -> ApiProvider {
        ApiProvider::Mxnzp
    }

    fn executor(&self) -> &QpsLimitedExecutor {
        &self.executor
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[tokio::test]
    async fn test_qps_limiting() {
        let provider = &*MXNZP_PROVIDER;

        log::info!(
            "Testing QPS limiting with {} QPS limit",
            provider.provider_type().qps_limit()
        );

        let start_time = Instant::now();

        // Make 3 consecutive requests
        for i in 0..3 {
            let request_start = Instant::now();
            let result = provider.get_latest_lottery().await;
            let request_duration = request_start.elapsed();

            log::debug!(
                "Request {}: {:?}, Duration: {:?}",
                i + 1,
                result.map(|_| "Success").unwrap_or("Failed"),
                request_duration
            );

            // After the first request, subsequent requests should be delayed
            if i > 0 {
                // Each request should take at least 1 second due to QPS limit of 1
                assert!(
                    request_duration >= std::time::Duration::from_millis(800),
                    "Request {} took only {:?}, expected at least 800ms due to QPS limiting",
                    i + 1,
                    request_duration
                );
            }
        }

        let total_duration = start_time.elapsed();
        log::debug!("Total duration for 3 requests: {:?}", total_duration);

        // Total time should be at least 2 seconds (1 second between each of the 3 requests)
        assert!(
            total_duration >= std::time::Duration::from_secs(2),
            "Total duration was {:?}, expected at least 2 seconds due to QPS limiting",
            total_duration
        );
    }
}
