use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::ipc::protocol::{AppState, RpcService};

#[derive(Clone)]
pub struct HttpServer {
    state: Arc<RwLock<AppState>>,
    addr: SocketAddr,
}

impl HttpServer {
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self::with_config(state, &HttpServerConfig::from_env())
    }

    pub fn with_config(state: Arc<RwLock<AppState>>, config: &HttpServerConfig) -> Self {
        Self {
            state,
            addr: config.socket_addr(),
        }
    }

    pub async fn start(&self) -> anyhow::Result<tokio::task::JoinHandle<()>> {
        let addr = self.addr;
        let app = build_router(self.state.clone());

        let listener = tokio::net::TcpListener::bind(addr).await?;
        log::info!("HTTP server listening on {addr}");

        let handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                log::error!("HTTP server stopped: {e}");
            }
        });

        Ok(handle)
    }
}

#[derive(Clone)]
struct RouterState {
    app_state: Arc<RwLock<AppState>>,
}

#[derive(Default)]
pub struct HttpServerConfig {
    pub host: String,
    pub port: u16,
}

impl HttpServerConfig {
    pub fn from_env() -> Self {
        let host = std::env::var("DBALL_HTTP_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
        let port = std::env::var("DBALL_HTTP_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(8081);
        Self { host, port }
    }

    pub fn socket_addr(&self) -> SocketAddr {
        let ip: IpAddr = self.host.parse().unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
        SocketAddr::new(ip, self.port)
    }
}

fn build_router(app_state: Arc<RwLock<AppState>>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/state", get(get_state))
        .route("/api/period/latest", get(get_latest_period))
        .route("/api/spots/unprized", get(get_unprized_spots))
        .route("/api/spots/prized", get(get_prized_spots))
        .route("/api/spots/update", post(update_all_unprize_spots))
        .route("/api/spots/deprecate", post(deprecate_last_batch_spots))
        .route("/api/spots/generate", post(generate_batch_spots))
        .route("/api/tickets/update-latest", post(update_latest_ticket))
        .route("/api/tickets/crawl", post(crawl_all_tickets))
        .route(
            "/api/tickets/update/periods",
            post(update_tickets_by_periods),
        )
        .route("/api/tickets/update/year", post(update_tickets_with_year))
        .route("/api/rpc", post(handle_rpc))
        .with_state(RouterState { app_state })
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    data: Option<Value>,
    error: Option<ApiError>,
}

#[derive(Serialize)]
struct ApiError {
    code: &'static str,
    message: String,
}

type ApiResult = (StatusCode, Json<ApiResponse>);

fn ok_value(value: Value) -> ApiResult {
    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: Some(value),
            error: None,
        }),
    )
}

fn err_response(status: StatusCode, code: &'static str, message: impl Into<String>) -> ApiResult {
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

async fn health() -> ApiResult {
    ok_value(json!({"status": "ok"}))
}

async fn get_state(State(state): State<RouterState>) -> ApiResult {
    let current = state.app_state.read().await.clone();
    match serde_json::to_value(current) {
        Ok(value) => ok_value(value),
        Err(e) => err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "serialize",
            e.to_string(),
        ),
    }
}

async fn get_latest_period(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::GetLatestPeriod, state).await
}

async fn get_unprized_spots(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::GetUnprizeSpots, state).await
}

async fn get_prized_spots(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::GetPrizedSpots, state).await
}

async fn update_all_unprize_spots(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::UpdateAllUnprizeSpots, state).await
}

async fn deprecate_last_batch_spots(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::DeprecatedLastBatchUnprizedSpot, state).await
}

async fn generate_batch_spots(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::GenerateBatchSpots, state).await
}

async fn update_latest_ticket(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::UpdateLatestTicket, state).await
}

async fn crawl_all_tickets(State(state): State<RouterState>) -> ApiResult {
    handle_rpc_service(RpcService::CrawlAllTickets, state).await
}

#[derive(Deserialize)]
struct PeriodsRequest {
    periods: Vec<String>,
}

async fn update_tickets_by_periods(
    State(state): State<RouterState>,
    Json(payload): Json<PeriodsRequest>,
) -> ApiResult {
    handle_rpc_service(RpcService::UpdateTicketsByPeriod(payload.periods), state).await
}

#[derive(Deserialize)]
struct YearRequest {
    year: i32,
}

async fn update_tickets_with_year(
    State(state): State<RouterState>,
    Json(payload): Json<YearRequest>,
) -> ApiResult {
    handle_rpc_service(RpcService::UpdateTicketsWithYear(payload.year), state).await
}

async fn handle_rpc(
    State(state): State<RouterState>,
    Json(service): Json<RpcService>,
) -> ApiResult {
    handle_rpc_service(service, state).await
}

async fn handle_rpc_service(service: RpcService, state: RouterState) -> ApiResult {
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

#[derive(Serialize)]
struct PeriodUpdateResult {
    period: String,
    inserted: Option<bool>,
    error: Option<String>,
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
