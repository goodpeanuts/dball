use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::api::{
    ApiCommon, CLIENT, MXNZP_PROVIDER,
    provider::{Provider as _, ProviderRequest, ProviderResponse},
};

impl crate::api::provider::mxnzp::MxnzpProvider {
    /// Execute latest lottery request with QPS limiting
    pub async fn get_latest_lottery(&self) -> anyhow::Result<GeneralLatestLotteryResponse> {
        let request = GeneralLatestLotteryRequest::new()?;

        self.execute_request(request).await
    }
}

static LATEST_TICKETS_API_COMMON: LazyLock<anyhow::Result<ApiCommon>> = LazyLock::new(|| {
    MXNZP_PROVIDER.create_api_common(
        crate::api::Protocol::Rest,
        &crate::api::provider::mxnzp::MxnzpApi::GetLatestLottery.to_string(),
    )
});

#[derive(Debug, Serialize)]
struct GeneralLatestLotteryRequest {
    app_id: String,
    app_secret: String,
    code: String,
}

impl GeneralLatestLotteryRequest {
    pub fn new() -> anyhow::Result<Self> {
        let (app_id, app_secret) = crate::api::MXNZP_PROVIDER.get_auth_config()?;

        let code = super::common::DEFAULT_LOTTERY_CODE;

        Ok(Self {
            app_id,
            app_secret,
            code: code.to_owned(),
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct GeneralLatestLotteryResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<super::common::LotteryData>,
}

impl ProviderResponse for GeneralLatestLotteryResponse {
    type Data = super::common::LotteryData;

    fn get_code(&self) -> i32 {
        self.code
    }

    fn get_msg(&self) -> String {
        self.msg.clone()
    }

    fn get_data(&self) -> Option<&Self::Data> {
        self.data.as_ref()
    }
}

impl ProviderRequest for GeneralLatestLotteryRequest {
    type Response = GeneralLatestLotteryResponse;

    async fn execute(self) -> anyhow::Result<Self::Response> {
        let common = LATEST_TICKETS_API_COMMON
            .as_ref()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let resp = CLIENT.get(common.url()).query(&self).send().await;

        let response = match resp {
            Ok(response) => {
                if response.status().is_success() {
                    response
                } else {
                    let error_message = format!(
                        "GeneralLatestLotteryRequest failed with status: {}",
                        response.status()
                    );
                    let text = response.text().await.unwrap_or_default();
                    log::error!("{error_message}\n==== Response: ====\n {text}");
                    return Err(anyhow::anyhow!("{error_message}"));
                }
            }
            Err(e) => return Err(anyhow::anyhow!("Request failed: {}", e)),
        };

        let response_text = response.text().await?;

        let api_response: GeneralLatestLotteryResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON response: {}", e))?;

        if api_response.code != crate::api::provider::mxnzp::RETURN_CODE_SUCCESS {
            return Err(anyhow::anyhow!("API returned error: {}", api_response.msg));
        }

        Ok(api_response)
    }
}
