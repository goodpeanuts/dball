use std::{str::FromStr, sync::LazyLock};

use crate::parse_from_env;

pub mod mxnzp;

pub use mxnzp::GENERAL_LATEST_LOTTERY_REQUEST;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[expect(async_fn_in_trait)]
pub trait SendRequest {
    async fn send(&self) -> anyhow::Result<reqwest::Response>;

    async fn resp_handle(
        &self,
        identifier: &str,
        resp: Result<reqwest::Response, reqwest::Error>,
    ) -> anyhow::Result<reqwest::Response> {
        match resp {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(response)
                } else {
                    let error_message =
                        format!("{} failed with status: {}", identifier, response.status());
                    let text = response.text().await.unwrap_or_default();
                    log::error!("{error_message}\n==== Response: ====\n {text}");
                    Err(anyhow::anyhow!("{error_message}"))
                }
            }
            Err(e) => Err(anyhow::anyhow!("Request failed: {}", e)),
        }
    }
}

pub(crate) struct RequestCommon {
    url: String,
    req_type: RequestType,
    return_type: RequestReturnType,
}

impl RequestCommon {
    /// Create a new `RequestCommon` with values from environment variables
    pub(crate) fn from_env(url_key: &str, req_type_key: &str, return_type_key: &str) -> Self {
        let url = parse_from_env(url_key);
        let req_type = parse_from_env::<RequestType>(req_type_key);
        let return_type = parse_from_env::<RequestReturnType>(return_type_key);

        Self {
            url,
            req_type,
            return_type,
        }
    }

    /// Create a new `RequestCommon` with explicit values
    #[expect(dead_code)]
    pub(crate) fn new(url: String, req_type: RequestType, return_type: RequestReturnType) -> Self {
        Self {
            url,
            req_type,
            return_type,
        }
    }

    pub(crate) fn url(&self) -> &String {
        &self.url
    }

    #[expect(dead_code)]
    pub(crate) fn req_type(&self) -> RequestType {
        self.req_type
    }

    #[expect(dead_code)]
    pub(crate) fn return_type(&self) -> RequestReturnType {
        self.return_type
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RequestType {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl FromStr for RequestType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "PATCH" => Ok(Self::Patch),
            _ => Err(format!("Invalid request type: {s}")),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum RequestReturnType {
    Json,
    Text,
    None,
}

impl FromStr for RequestReturnType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "JSON" => Ok(Self::Json),
            "TEXT" => Ok(Self::Text),
            "NONE" => Ok(Self::None),
            _ => Err(format!("Invalid return type: {s}")),
        }
    }
}
