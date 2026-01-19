use std::sync::Arc;

use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use aide::openapi::{Info, OpenApi};
use aide::scalar::Scalar;
use axum::{Extension, Json, Router, routing::get as axum_get};
use tokio::sync::RwLock;

use crate::ipc::protocol::AppState;

use super::handlers::{
    crawl_all_tickets, deprecate_last_batch_spots, generate_batch_spots, get_latest_period,
    get_prized_spots, get_state, get_unprized_spots, handle_rpc, health, update_all_unprize_spots,
    update_latest_ticket, update_tickets_by_periods, update_tickets_with_year,
};
use super::types::RouterState;

pub(super) fn build_router(app_state: Arc<RwLock<AppState>>) -> Router {
    let mut api = OpenApi {
        info: Info {
            title: "DBall HTTP API".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            ..Default::default()
        },
        ..Default::default()
    };

    let app = ApiRouter::new()
        .route(
            "/api/docs",
            Scalar::new("/api/docs/openapi.json")
                .with_title("DBall API Docs")
                .axum_route(),
        )
        .api_route("/health", get(health))
        .api_route("/api/state", get(get_state))
        .api_route("/api/period/latest", get(get_latest_period))
        .api_route("/api/spots/unprized", get(get_unprized_spots))
        .api_route("/api/spots/prized", get(get_prized_spots))
        .api_route("/api/spots/update", post(update_all_unprize_spots))
        .api_route("/api/spots/deprecate", post(deprecate_last_batch_spots))
        .api_route("/api/spots/generate", post(generate_batch_spots))
        .api_route("/api/tickets/update-latest", post(update_latest_ticket))
        .api_route("/api/tickets/crawl", post(crawl_all_tickets))
        .api_route(
            "/api/tickets/update/periods",
            post(update_tickets_by_periods),
        )
        .api_route("/api/tickets/update/year", post(update_tickets_with_year))
        .api_route("/api/rpc", post(handle_rpc))
        .with_state(RouterState { app_state })
        .finish_api(&mut api);

    let api = Arc::new(api);
    app.route("/api/docs/openapi.json", axum_get(serve_openapi))
        .layer(Extension(api))
}

async fn serve_openapi(Extension(api): Extension<Arc<OpenApi>>) -> Json<OpenApi> {
    Json((*api).clone())
}
