use std::sync::Arc;

use axum::http::StatusCode;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::ipc::protocol::{AppState, RpcService};

use super::types::{ApiResult, PeriodUpdateResult, RouterState, err_response, ok_value};

pub(super) async fn handle_rpc_service(service: RpcService, state: RouterState) -> ApiResult {
    match dispatch_rpc(service, state.app_state).await {
        Ok(value) => ok_value(value),
        Err(err) => err_response(err.status, err.code, err.message),
    }
}

struct ApiFailure {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiFailure {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "bad_request",
            message: message.into(),
        }
    }

    fn not_supported(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_IMPLEMENTED,
            code: "not_supported",
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: message.into(),
        }
    }
}

async fn dispatch_rpc(
    service: RpcService,
    state: Arc<RwLock<AppState>>,
) -> Result<Value, ApiFailure> {
    match service {
        RpcService::GetCurrentState => {
            let current = state.read().await.clone();
            serde_json::to_value(current).map_err(|e| ApiFailure::internal(e.to_string()))
        }
        RpcService::UpdateLatestTicket => {
            let ticket = crate::service::update_latest_ticket()
                .await
                .map_err(|e| ApiFailure::internal(e.to_string()))?;
            serde_json::to_value(ticket).map_err(|e| ApiFailure::internal(e.to_string()))
        }
        RpcService::GetLatestPeriod => {
            let period = crate::service::get_next_period()
                .await
                .map_err(|e| ApiFailure::internal(e.to_string()))?;
            Ok(Value::String(period))
        }
        RpcService::UpdateAllUnprizeSpots => {
            let spots = crate::service::update_all_unprize_spots()
                .await
                .map_err(|e| ApiFailure::internal(e.to_string()))?;
            serde_json::to_value(spots).map_err(|e| ApiFailure::internal(e.to_string()))
        }
        RpcService::DeprecatedLastBatchUnprizedSpot => {
            let count = crate::service::deprecated_last_batch_unprized_spot()
                .await
                .map_err(|e| ApiFailure::internal(e.to_string()))?;
            serde_json::to_value(count).map_err(|e| ApiFailure::internal(e.to_string()))
        }
        RpcService::GetUnprizeSpots => {
            let spots = crate::service::get_next_period_unprized_spots()
                .await
                .map_err(|e| ApiFailure::internal(e.to_string()))?;
            serde_json::to_value(spots).map_err(|e| ApiFailure::internal(e.to_string()))
        }
        RpcService::GetPrizedSpots => {
            let spots = crate::service::get_prized_spots()
                .await
                .map_err(|e| ApiFailure::internal(e.to_string()))?;
            serde_json::to_value(spots).map_err(|e| ApiFailure::internal(e.to_string()))
        }
        RpcService::GenerateBatchSpots => {
            crate::service::generate_batch_spots()
                .await
                .map_err(|e| ApiFailure::internal(e.to_string()))?;
            Ok(Value::Null)
        }
        RpcService::CrawlAllTickets => {
            crate::service::crawl_all_tickets()
                .await
                .map_err(|e| ApiFailure::internal(e.to_string()))?;
            Ok(Value::Null)
        }
        RpcService::UpdateTicketsByPeriod(periods) => {
            if periods.is_empty() {
                return Err(ApiFailure::bad_request("periods must not be empty"));
            }
            let mut results = Vec::with_capacity(periods.len());
            for period in periods {
                match crate::service::update_tickets_by_period(&period).await {
                    Ok(inserted) => results.push(PeriodUpdateResult {
                        period,
                        inserted: Some(inserted),
                        error: None,
                    }),
                    Err(e) => results.push(PeriodUpdateResult {
                        period,
                        inserted: None,
                        error: Some(e.to_string()),
                    }),
                }
            }
            serde_json::to_value(results).map_err(|e| ApiFailure::internal(e.to_string()))
        }
        RpcService::UpdateTicketsWithYear(year) => {
            if year <= 0 {
                return Err(ApiFailure::bad_request("year must be positive"));
            }
            crate::service::update_tickets_with_year(year as usize)
                .await
                .map_err(|e| ApiFailure::internal(e.to_string()))?;
            Ok(Value::Null)
        }
        RpcService::Shutdown | RpcService::Restart => Err(ApiFailure::not_supported(
            "operation is not supported via HTTP",
        )),
    }
}
