use anyhow::{Result, anyhow};
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{RwLock, broadcast};
use tokio::task::JoinHandle;

use crate::ipc::{
    codec::{FrameBuffer, IpcCodec},
    envelope::{IpcEnvelope, IpcKind},
    protocol::{AppState, ErrorMessage, HelloMessage, RpcService},
};

/// IPC Server
/// Provides an asynchronous IPC server using Unix Domain Sockets
pub struct IpcServer {
    state: Arc<RwLock<AppState>>,

    state_broadcaster: broadcast::Sender<AppState>,

    socket_path: String,
}

impl IpcServer {
    /// Unix Domain Socket path
    #[cfg(unix)]
    const SOCKET_PATH: &'static str = "/tmp/dball-daemon.sock";

    /// Windows Named Pipe name
    #[cfg(windows)]
    const PIPE_NAME: &'static str = r"\\.\pipe\dball-daemon";

    /// Create a new IPC server
    pub async fn new(
        state: Arc<RwLock<AppState>>,
        state_broadcaster: broadcast::Sender<AppState>,
    ) -> Result<Self> {
        #[cfg(unix)]
        let socket_path = Self::SOCKET_PATH.to_owned();

        #[cfg(windows)]
        let socket_path = Self::PIPE_NAME.to_string();

        Ok(Self {
            state,
            state_broadcaster,
            socket_path,
        })
    }

    /// 启动IPC服务器
    pub async fn start(&self) -> Result<JoinHandle<()>> {
        #[cfg(unix)]
        {
            self.start_unix_server().await
        }

        #[cfg(windows)]
        {
            // TODO: 实现Windows Named Pipe版本
            todo!("Windows Named Pipe support not implemented yet")
        }
    }

    /// 启动Unix Domain Socket服务器
    #[cfg(unix)]
    async fn start_unix_server(&self) -> Result<JoinHandle<()>> {
        // 清理可能存在的旧socket文件
        if Path::new(&self.socket_path).exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;

        let state = self.state.clone();
        let state_broadcaster = self.state_broadcaster.clone();

        log::info!("IPC server listening on {}", self.socket_path);

        let handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let state = state.clone();
                        let state_broadcaster = state_broadcaster.clone();

                        tokio::spawn(async move {
                            if let Err(e) =
                                Self::handle_client(stream, state, state_broadcaster).await
                            {
                                log::error!("Client handler error: {e}");
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to accept connection: {e}");
                        break;
                    }
                }
            }
        });

        Ok(handle)
    }

    async fn handle_client(
        mut stream: UnixStream,
        state: Arc<RwLock<AppState>>,
        state_broadcaster: broadcast::Sender<AppState>,
    ) -> Result<()> {
        log::info!("New client connected");

        let mut buffer = FrameBuffer::new();
        let mut read_buf = vec![0u8; 4096];
        let mut state_receiver = state_broadcaster.subscribe();

        loop {
            tokio::select! {
                result = stream.read(&mut read_buf) => {
                    match result {
                        Ok(0) => {
                            log::info!("Client disconnected");
                            break;
                        }
                        Ok(n) => {
                            buffer.push(&read_buf[0..n]);

                            // try to decode messages
                            while let Some(envelope) = buffer.try_decode::<serde_json::Value>()? {
                                if let Err(e) = Self::process_message(envelope, &mut stream, &state).await {
                                    log::error!("Failed to process message: {e}");
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to read from client: {e}");
                            break;
                        }
                    }
                }

                // broadcast state updates
                result = state_receiver.recv() => {
                    match result {
                        Ok(new_state) => {
                            let event_envelope = IpcEnvelope::new(
                                IpcKind::Event,
                                serde_json::to_value(&new_state)?
                            );

                            if let Err(e) = Self::send_message(&mut stream, &event_envelope).await {
                                log::error!("Failed to send state update: {e}");
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            log::warn!("Client lagged behind on state updates");
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            log::info!("State broadcaster closed");
                            break;
                        }
                    }
                }
            }
        }

        log::info!("Client handler finished");
        Ok(())
    }

    /// Process incoming messages from the client
    async fn process_message(
        envelope: IpcEnvelope,
        stream: &mut UnixStream,
        state: &Arc<RwLock<AppState>>,
    ) -> Result<()> {
        match &envelope.kind {
            IpcKind::Hello => Self::handle_hello(envelope, stream).await,
            IpcKind::Subscribe => Self::handle_subscribe(envelope, stream, state).await,
            IpcKind::Request(_rpc_service) => Self::handle_request(envelope, stream, state).await,
            _ => {
                log::warn!("Unexpected message kind: {:?}", envelope.kind);
                Ok(())
            }
        }
    }

    /// Process Hello message from the client
    async fn handle_hello(envelope: IpcEnvelope, stream: &mut UnixStream) -> Result<()> {
        log::info!("Received Hello message from client");

        // 创建Hello响应
        let hello_response = HelloMessage {
            version: 1,
            client_info: None,
            server_name: Some("dball-daemon".to_owned()),
            supported_features: vec![
                "basic_rpc".to_owned(),
                "state_subscription".to_owned(),
                "compression".to_owned(),
            ],
        };

        let response_envelope = IpcEnvelope::new_with_uuid(
            IpcKind::Hello,
            serde_json::to_value(hello_response)?,
            envelope.uuid,
        );

        Self::send_message(stream, &response_envelope).await
    }

    /// Process Subscribe message from the client
    async fn handle_subscribe(
        envelope: IpcEnvelope,
        stream: &mut UnixStream,
        state: &Arc<RwLock<AppState>>,
    ) -> Result<()> {
        log::info!("Received Subscribe message from client");
        let current_state = state.read().await.clone();

        let response = IpcEnvelope::new_with_uuid(
            IpcKind::Response,
            serde_json::to_value(&current_state)?,
            envelope.uuid,
        );

        Self::send_message(stream, &response).await
    }

    /// Get current application state
    #[expect(unused)]
    async fn get_current_state(&self) -> Result<AppState> {
        let state = self.state.read().await;
        Ok(state.clone())
    }

    /// Process and send message to the client
    async fn send_message(stream: &mut UnixStream, envelope: &IpcEnvelope) -> Result<()> {
        let encoded = IpcCodec::encode(envelope)?;
        stream.write_all(&encoded).await?;
        Ok(())
    }

    #[expect(unused)]
    async fn send_error(
        stream: &mut UnixStream,
        request_uuid: String,
        code: u32,
        message: String,
    ) -> Result<()> {
        let error_msg = ErrorMessage {
            code,
            message,
            details: None,
        };

        let error_envelope = IpcEnvelope::new_with_uuid(
            IpcKind::Err,
            serde_json::to_value(error_msg)?,
            request_uuid,
        );

        Self::send_message(stream, &error_envelope).await
    }

    /// Process RPC request from the client
    #[expect(clippy::too_many_lines)]
    async fn handle_request(
        envelope: IpcEnvelope,
        stream: &mut UnixStream,
        state: &Arc<RwLock<AppState>>,
    ) -> Result<()> {
        log::debug!(
            "Received RPC {} request from client, uuid: {}",
            envelope.kind,
            envelope.uuid
        );

        match envelope.kind {
            IpcKind::Request(service) => {
                match service {
                    RpcService::GetCurrentState => {
                        let current_state = state.read().await.clone();
                        let response = IpcEnvelope::new_with_uuid(
                            IpcKind::Response,
                            serde_json::to_value(current_state)?,
                            envelope.uuid,
                        );
                        Self::send_message(stream, &response).await
                    }
                    RpcService::UpdateLatestTicket => {
                        let ticket = crate::service::update_latest_ticket()
                            .await
                            .map_err(|e| e.to_string());
                        let response = IpcEnvelope::new_with_uuid(
                            IpcKind::Response,
                            serde_json::to_value(ticket)?,
                            envelope.uuid,
                        );
                        Self::send_message(stream, &response).await
                    }
                    RpcService::GetLatestPeriod => {
                        let next_period = crate::service::get_next_period()
                            .await
                            .map_err(|e| e.to_string());
                        let response = IpcEnvelope::new_with_uuid(
                            IpcKind::Response,
                            serde_json::to_value(next_period)?,
                            envelope.uuid,
                        );
                        Self::send_message(stream, &response).await
                    }
                    RpcService::UpdateAllUnprizeSpots => {
                        let state = crate::service::update_all_unprize_spots()
                            .await
                            .map_err(|e| e.to_string());
                        let response = IpcEnvelope::new_with_uuid(
                            IpcKind::Response,
                            serde_json::to_value(state)?,
                            envelope.uuid,
                        );

                        Self::send_message(stream, &response).await
                    }
                    RpcService::DeprecatedLastBatchUnprizedSpot => {
                        let result = crate::service::deprecated_last_batch_unprized_spot()
                            .await
                            .map_err(|e| e.to_string());
                        let response = IpcEnvelope::new_with_uuid(
                            IpcKind::Response,
                            serde_json::to_value(result)?,
                            envelope.uuid,
                        );

                        Self::send_message(stream, &response).await
                    }
                    RpcService::GetUnprizeSpots => {
                        let dballs = crate::service::get_next_period_unprized_spots()
                            .await
                            .map_err(|e| e.to_string());
                        let response = IpcEnvelope::new_with_uuid(
                            IpcKind::Response,
                            serde_json::to_value(dballs)?,
                            envelope.uuid,
                        );
                        Self::send_message(stream, &response).await
                    }
                    RpcService::GetPrizedSpots => {
                        let dballs = crate::service::get_prized_spots()
                            .await
                            .map_err(|e| e.to_string());
                        let response = IpcEnvelope::new_with_uuid(
                            IpcKind::Response,
                            serde_json::to_value(dballs)?,
                            envelope.uuid,
                        );
                        Self::send_message(stream, &response).await
                    }
                    RpcService::GenerateBatchSpots => {
                        let result = crate::service::generate_batch_spots()
                            .await
                            .map_err(|e| e.to_string());
                        let response = IpcEnvelope::new_with_uuid(
                            IpcKind::Response,
                            serde_json::to_value(result)?,
                            envelope.uuid,
                        );
                        Self::send_message(stream, &response).await
                    }
                    _ => {
                        // other RPC services are not implemented yet
                        let response = IpcEnvelope::new(
                            IpcKind::Response,
                            serde_json::to_value(format!(
                                "RPC service {service:?} not implemented yet"
                            ))?,
                        );
                        Self::send_message(stream, &response).await
                    }
                }
            }
            _ => Err(anyhow!("Expected Request, got {:?}", envelope.kind)),
        }
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        // Cleanup socket file on Unix systems
        #[cfg(unix)]
        if Path::new(&self.socket_path).exists() {
            if let Err(e) = std::fs::remove_file(&self.socket_path) {
                log::error!("Failed to cleanup socket file: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_ipc_server_creation() {
        let initial_state = AppState {
            current_period: "test".to_owned(),
            next_period: "test".to_owned(),
            last_draw_time: None,
            next_draw_time: None,
            latest_ticket: None,
            pending_tickets: vec![],
            unprize_spots_count: 0,
            total_investment: 0.0,
            total_return: 0.0,
            api_status: crate::ipc::protocol::ApiStatusInfo {
                api_provider: "test".to_owned(),
                last_success: None,
                success_rate: 0.0,
                average_response_time: Duration::from_millis(1000),
            },
            last_update: chrono::Utc::now(),
            daemon_uptime: Duration::from_secs(0),
            generation_status: crate::ipc::protocol::GenerationStatus::Idle,
            last_generation_time: None,
        };

        let state = Arc::new(RwLock::new(initial_state));
        let (broadcaster, _) = broadcast::channel(10);

        let server = IpcServer::new(state, broadcaster).await;
        assert!(server.is_ok());
    }
}
