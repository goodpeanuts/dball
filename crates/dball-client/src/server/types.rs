use std::sync::Arc;

use axum::{Json, http::StatusCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;

use crate::ipc::protocol::AppState;

#[derive(Clone)]
pub(super) struct RouterState {
    pub(super) app_state: Arc<RwLock<AppState>>,
}

#[derive(Serialize, JsonSchema)]
pub(super) struct ApiResponse {
    success: bool,
    data: Option<Value>,
    error: Option<ApiError>,
}

#[derive(Serialize, JsonSchema)]
pub(super) struct ApiError {
    code: &'static str,
    message: String,
}

pub(super) type ApiResult = (StatusCode, Json<ApiResponse>);

pub(super) fn ok_value(value: Value) -> ApiResult {
    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: Some(value),
            error: None,
        }),
    )
}

pub(super) fn err_response(
    status: StatusCode,
    code: &'static str,
    message: impl Into<String>,
) -> ApiResult {
    (
        status,
        Json(ApiResponse {
            success: false,
            data: None,
            error: Some(ApiError {
                code,
                message: message.into(),
            }),
        }),
    )
}

#[derive(Deserialize, JsonSchema)]
pub(super) struct PeriodsRequest {
    pub(super) periods: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
pub(super) struct YearRequest {
    pub(super) year: i32,
}

#[derive(Serialize, JsonSchema)]
pub(super) struct PeriodUpdateResult {
    pub(super) period: String,
    pub(super) inserted: Option<bool>,
    pub(super) error: Option<String>,
}
