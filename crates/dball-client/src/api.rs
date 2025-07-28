use std::{str::FromStr, sync::LazyLock};

pub mod config;
pub mod fix;
pub mod provider;
pub mod rest;
pub mod websocket;

pub use provider::mxnzp::MXNZP_PROVIDER;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Deserialize, Serialize)]
pub enum Protocol {
    #[strum(to_string = "rest")]
    Rest,
    #[strum(to_string = "websocket")]
    WebSocket,
    #[strum(to_string = "fix")]
    Fix,
    #[strum(to_string = "mq")]
    MQ,
    #[strum(to_string = "grpc")]
    Grpc,
}

impl FromStr for Protocol {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "REST" => Ok(Self::Rest),
            "WEBSOCKET" => Ok(Self::WebSocket),
            "FIX" => Ok(Self::Fix),
            "MQ" => Ok(Self::MQ),
            "GRPC" => Ok(Self::Grpc),
            _ => Err(format!("Invalid protocol: {s}")),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiCommon {
    pub name: String,
    pub protocol: Protocol,
    pub url: String,
    pub timeout_ms: Option<usize>,
}

impl ApiCommon {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn protocol(&self) -> Protocol {
        self.protocol
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
