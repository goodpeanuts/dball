use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::{
    parse_from_env,
    request::{
        Api, ApiCommon, CLIENT,
        provider::{ProviderRequest, ProviderResponse},
    },
};

pub static GENERAL_LATEST_LOTTERY_API_COMMON: LazyLock<ApiCommon> = LazyLock::new(|| {
    ApiCommon::from_env(
        "GENERAL_LATEST_LOTTERY_URL",
        "GENERAL_LATEST_LOTTERY_METHOD",
        "GENERAL_LATEST_LOTTERY_RETURN_TYPE",
    )
});

pub fn create_lottery_request() -> Api<GeneralLatestLotteryRequest> {
    Api {
        common: &GENERAL_LATEST_LOTTERY_API_COMMON,
        state: GeneralLatestLotteryRequest::new(),
    }
}

#[derive(Debug)]
pub struct GeneralLatestLotteryRequest {
    params: GeneralLatestLotteryRequestParams,
}

#[derive(Debug, Serialize)]
struct GeneralLatestLotteryRequestParams {
    app_id: String,
    app_secret: String,
    code: String,
}

impl ProviderRequest for GeneralLatestLotteryRequest {
    type Response = GeneralLatestLotteryResponse;

    async fn execute(self, common: &ApiCommon) -> anyhow::Result<Self::Response> {
        let params = &self.params;

        let resp = CLIENT.get(common.url()).query(params).send().await;

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

        if api_response.code != 1 {
            return Err(anyhow::anyhow!("API returned error: {}", api_response.msg));
        }

        Ok(api_response)
    }
}

impl GeneralLatestLotteryRequest {
    pub fn new() -> Self {
        let app_id = parse_from_env::<String>("APP_ID");
        let app_secret = parse_from_env::<String>("APP_SECRET");

        Self {
            params: GeneralLatestLotteryRequestParams {
                app_id,
                app_secret,
                code: super::common::DEFAULT_LOTTERY_CODE.to_owned(),
            },
        }
    }

    #[cfg(test)]
    pub fn new_with_params(app_id: String, app_secret: String, code: String) -> Self {
        Self {
            params: GeneralLatestLotteryRequestParams {
                app_id,
                app_secret,
                code,
            },
        }
    }
}

impl Default for GeneralLatestLotteryRequest {
    fn default() -> Self {
        Self::new()
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
