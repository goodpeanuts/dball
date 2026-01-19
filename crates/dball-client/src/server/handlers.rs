use axum::{Json, extract::State};
use serde_json::json;

use crate::ipc::protocol::RpcService;

use super::rpc::handle_rpc_service;
use super::types::{ApiResult, PeriodsRequest, RouterState, YearRequest, err_response, ok_value};

pub(super) async fn health() -> ApiResult {
    ok_value(json!({"status": "ok"}))
}

pub(super) async fn get_state(State(state): State<RouterState>) -> ApiResult {
    let current = state.app_state.read().await.clone();
    match serde_json::to_value(current) {
        Ok(value) => ok_value(value),
        Err(e) => err_response(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "serialize",
            e.to_string(),
        ),
    }
}

pub(super) async fn get_latest_period(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::GetLatestPeriod, state).await
}

pub(super) async fn get_unprized_spots(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::GetUnprizeSpots, state).await
}

pub(super) async fn get_prized_spots(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::GetPrizedSpots, state).await
}

pub(super) async fn update_all_unprize_spots(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::UpdateAllUnprizeSpots, state).await
}

pub(super) async fn deprecate_last_batch_spots(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::DeprecatedLastBatchUnprizedSpot, state).await
}

pub(super) async fn generate_batch_spots(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::GenerateBatchSpots, state).await
}

pub(super) async fn update_latest_ticket(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::UpdateLatestTicket, state).await
}

pub(super) async fn crawl_all_tickets(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::CrawlAllTickets, state).await
}

pub(super) async fn update_tickets_by_periods(
    State(state): State<RouterState>,
    Json(payload): Json<PeriodsRequest>,
) -> ApiResult {
    handle_rpc_service(RpcService::UpdateTicketsByPeriod(payload.periods), state).await
}

pub(super) async fn update_tickets_with_year(
    State(state): State<RouterState>,
    Json(payload): Json<YearRequest>,
) -> ApiResult {
    handle_rpc_service(RpcService::UpdateTicketsWithYear(payload.year), state).await
}

pub(super) async fn handle_rpc(
    State(state): State<RouterState>,
    Json(service): Json<RpcService>,
) -> ApiResult {
    handle_rpc_service(service, state).await
}
