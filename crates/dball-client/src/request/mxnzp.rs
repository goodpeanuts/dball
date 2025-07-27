use crate::{impl_api_factory, impl_provider, request::factory::ApiFactory as _};
use latest_ticket::GeneralLatestLotteryResponse;
use latest_ticket::create_lottery_request;
use specified_ticket::GeneralSpecifiedLotteryResponse;
use specified_ticket::create_specified_lottery_request;

pub mod common;
pub mod latest_ticket;
pub mod specified_ticket;

/// Global MXNZP provider instance
pub static MXNZP_PROVIDER: MxnzpProvider = MxnzpProvider;

impl_provider!(MxnzpProvider, crate::request::provider::ApiProvider::Mxnzp);
impl_api_factory!(MxnzpProvider);

/// MXNZP API provider with QPS limit of 1
#[derive(Debug, Clone, Copy)]
pub struct MxnzpProvider;

impl MxnzpProvider {
    /// Create a new MXNZP provider instance
    pub fn new() -> Self {
        Self
    }

    /// Execute latest lottery request with QPS limiting
    pub async fn get_latest_lottery(&self) -> anyhow::Result<GeneralLatestLotteryResponse> {
        let request = create_lottery_request().state;
        let factory_request = self.create_request(request);

        // Get the common config from the original implementation
        factory_request
            .execute(&latest_ticket::GENERAL_LATEST_LOTTERY_API_COMMON)
            .await
    }

    /// Execute specified lottery request with QPS limiting
    pub async fn get_specified_lottery(
        &self,
        expect: &str,
    ) -> anyhow::Result<GeneralSpecifiedLotteryResponse> {
        let request = create_specified_lottery_request(expect.to_owned()).state;
        let factory_request = self.create_request(request);

        // Get the common config from the original implementation
        factory_request
            .execute(&specified_ticket::GENERAL_SPECIFIED_LOTTERY_API_COMMON)
            .await
    }
}

impl Default for MxnzpProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::models::Ticket;
    use crate::request::provider::{Provider, ProviderResponse};
    use std::time::Instant;

    use super::*;

    #[tokio::test]
    async fn test_mxnzp_latest_lottery() {
        // QPS: 1
        let resp = MXNZP_PROVIDER.get_latest_lottery().await;

        if let Ok(response) = resp {
            if let Some(data) = response.data {
                let ticket = Ticket::try_from(data);
                assert!(ticket.is_ok(), "Failed to convert LotteryData to Ticket");
            } else {
                panic!("Failed to get latest lottery");
            };
        }
    }

    #[tokio::test]
    async fn test_mxnzp_specified_lottery() {
        // QPS: 1

        let expect = "2025084";
        let resp = MXNZP_PROVIDER.get_specified_lottery(expect).await;

        if let Ok(response) = resp {
            if let Some(data) = response.get_data() {
                log::debug!("API Response data: {data:?}");
                let ticket = Ticket::try_from(data);
                assert!(ticket.is_ok(), "Failed to convert LotteryData to Ticket");

                if let Ok(ticket) = ticket {
                    log::debug!("Converted ticket: {ticket}");
                }
            } else {
                panic!("Failed to get specified lottery");
            };
        } else if let Err(e) = resp {
            panic!("API request failed: {e}");
        }
    }

    #[tokio::test]
    async fn test_qps_limiting() {
        let provider = &MXNZP_PROVIDER;

        log::info!(
            "Testing QPS limiting with {} QPS limit",
            provider.provider_id().qps_limit()
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
