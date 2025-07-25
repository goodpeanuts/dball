use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::{models::Ticket, parse_from_env};

#[derive(Debug)]
pub struct GeneralLatestLotteryRequest {
    params: QueryParams,
}

#[derive(Debug, Serialize)]
struct QueryParams {
    app_id: String,
    app_secret: String,
    code: String,
}

#[derive(Debug, Deserialize)]
pub struct GeneralLatestLotteryResponse {
    pub code: i32,
    pub msg: String,
    pub data: Option<LotteryData>,
}

#[derive(Debug, Deserialize)]
pub struct LotteryData {
    #[serde(rename = "openCode")]
    pub open_code: String,
    pub code: String,
    #[serde(rename = "expect")]
    pub period: String,
    pub name: String,
    pub time: String,
}

impl TryFrom<LotteryData> for Ticket {
    type Error = anyhow::Error;

    fn try_from(data: LotteryData) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = data.open_code.split('+').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid open_code format: {}",
                data.open_code
            ));
        }

        let red_balls: Result<Vec<i32>, _> = parts[0]
            .split(',')
            .map(|s| s.trim().parse::<i32>())
            .collect();

        let red_balls =
            red_balls.map_err(|e| anyhow::anyhow!("Failed to parse red balls: {}", e))?;

        let blue_ball: i32 = parts[1]
            .trim()
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse blue ball: {}", e))?;

        Ok(Self::new(data.period, &data.time, &red_balls, blue_ball)?)
    }
}

pub static GENERAL_LATEST_LOTTERY_API_COMMON: LazyLock<super::ApiCommon> = LazyLock::new(|| {
    super::ApiCommon::from_env(
        "GENERAL_LATEST_LOTTERY_URL",
        "GENERAL_LATEST_LOTTERY_METHOD",
        "GENERAL_LATEST_LOTTERY_RETURN_TYPE",
    )
});

pub fn create_lottery_request() -> super::Api<GeneralLatestLotteryRequest> {
    super::Api {
        common: &GENERAL_LATEST_LOTTERY_API_COMMON,
        state: GeneralLatestLotteryRequest::new(),
    }
}

pub async fn get_latest_lottery() -> anyhow::Result<GeneralLatestLotteryResponse> {
    create_lottery_request().execute().await
}

impl super::ApiRequest for GeneralLatestLotteryRequest {
    type Response = GeneralLatestLotteryResponse;

    async fn execute(self, common: &super::ApiCommon) -> anyhow::Result<Self::Response> {
        let params = &self.params;

        let resp = super::CLIENT.get(common.url()).query(params).send().await;

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
            params: QueryParams {
                app_id,
                app_secret,
                code: "ssq".to_owned(),
            },
        }
    }

    #[cfg(test)]
    pub fn new_with_params(app_id: String, app_secret: String, code: String) -> Self {
        Self {
            params: QueryParams {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mxnzp_latest_lottery() {
        let resp = get_latest_lottery().await;

        if let Ok(response) = resp {
            if let Some(data) = response.data {
                let ticket = Ticket::try_from(data);
                assert!(ticket.is_ok(), "Failed to convert LotteryData to Ticket");
            } else {
                panic!("Failed to get latest lottery");
            };
        }
    }
}
