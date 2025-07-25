use std::{str::FromStr, sync::LazyLock};

use crate::parse_from_env;

pub mod latest_ticket;
pub mod specified_ticket;

pub use latest_ticket::create_lottery_request;
use serde::Deserialize;
pub use specified_ticket::{create_specified_lottery_request, get_specified_lottery};

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[derive(Debug, Clone)]
pub struct ApiCommon {
    url: String,
    req_type: RequestType,
    return_type: RequestReturnType,
}

pub struct Api<S>
where
    S: ApiRequest,
{
    pub common: &'static ApiCommon,
    pub state: S,
}

impl<S> Api<S>
where
    S: ApiRequest,
{
    pub async fn execute(self) -> anyhow::Result<S::Response> {
        self.state.execute(self.common).await
    }
}

#[expect(async_fn_in_trait)]
pub trait ApiRequest
where
    for<'de> Self::Response: Deserialize<'de>,
{
    type Response;

    async fn execute(self, common: &ApiCommon) -> anyhow::Result<Self::Response>;
}

impl ApiCommon {
    /// Create a new `ApiCommon` with values from environment variables
    pub fn from_env(url_key: &str, req_type_key: &str, return_type_key: &str) -> Self {
        let url = parse_from_env(url_key);
        let req_type = parse_from_env::<RequestType>(req_type_key);
        let return_type = parse_from_env::<RequestReturnType>(return_type_key);

        Self {
            url,
            req_type,
            return_type,
        }
    }

    /// Create a new `ApiCommon` with explicit values
    pub fn new(url: String, req_type: RequestType, return_type: RequestReturnType) -> Self {
        Self {
            url,
            req_type,
            return_type,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn req_type(&self) -> RequestType {
        self.req_type
    }

    pub fn return_type(&self) -> RequestReturnType {
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
