use std::sync::LazyLock;

use serde::Serialize;

use crate::{init_env, parse_from_env};

pub static GENERAL_LATEST_LOTTERY_REQUEST: LazyLock<GeneralLatestLottery> = LazyLock::new(|| {
    init_env();

    GeneralLatestLottery::new(super::RequestCommon::from_env(
        "GENERAL_LATEST_LOTTERY_URL",
        "GENERAL_LATEST_LOTTERY_METHOD",
        "GENERAL_LATEST_LOTTERY_RETURN_TYPE",
    ))
});

pub struct GeneralLatestLottery {
    common: super::RequestCommon,

    params: QueryParams,
}

#[derive(Serialize)]
struct QueryParams {
    app_id: String,
    app_secret: String,
    code: String,
}

impl super::SendRequest for GeneralLatestLottery {
    async fn send(&self) -> anyhow::Result<reqwest::Response> {
        let params = &self.params;

        let resp = super::CLIENT
            .get(self.common.url())
            .query(params)
            .send()
            .await;

        self.resp_handle("GeneralLatestLottery", resp).await
    }
}

impl GeneralLatestLottery {
    pub(crate) fn new(common: super::RequestCommon) -> Self {
        let app_id = parse_from_env::<String>("APP_ID");
        let app_secret = parse_from_env::<String>("APP_SECRET");

        Self {
            common,
            params: QueryParams {
                app_id,
                app_secret,
                code: "ssq".to_owned(),
            },
        }
    }
}
