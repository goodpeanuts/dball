use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::ipc::protocol::AppState;

mod handlers;
mod router;
mod rpc;
mod types;

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
        let app = router::build_router(self.state.clone());

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
