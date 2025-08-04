use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::UnixStream;
use tokio::sync::{RwLock, mpsc, oneshot};

use crate::ipc::{
    codec::{FrameBuffer, IpcCodec},
    envelope::{IpcEnvelope, IpcKind},
    protocol::{AppState, EventType, HelloMessage, SubscribeMessage},
};

#[derive(Debug, Clone)]
pub enum ClientState {
    Disconnected,
    Connecting,
    Connected,
    Authenticated,
    Subscribed,
    Error(String),
}

/// IPC Client
///
/// Communicates with the daemon process using IPC protocols
pub struct IpcClient {
    /// Client state
    state: Arc<RwLock<ClientState>>,
    /// Socket path
    socket_path: String,
    /// Current application state
    app_state: Arc<RwLock<Option<AppState>>>,
    /// Message sender channel
    message_sender: Option<mpsc::UnboundedSender<IpcEnvelope>>,
    /// Pending requests waiting for responses
    pending_requests: Arc<RwLock<HashMap<String, oneshot::Sender<serde_json::Value>>>>,
}

impl IpcClient {
    /// Unix Domain Socket path
    #[cfg(unix)]
    const SOCKET_PATH: &'static str = "/tmp/dball-daemon.sock";

    /// Windows Named Pipe name
    #[cfg(windows)]
    const PIPE_NAME: &'static str = r"\\.\pipe\dball-daemon";

    /// Create a new IPC client
    pub fn new() -> Self {
        #[cfg(unix)]
        let socket_path = Self::SOCKET_PATH.to_owned();

        #[cfg(windows)]
        let socket_path = Self::PIPE_NAME.to_string();

        Self {
            state: Arc::new(RwLock::new(ClientState::Disconnected)),
            socket_path,
            app_state: Arc::new(RwLock::new(None)),
            message_sender: None,
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn new_connected() -> Result<Self> {
        let mut client = Self::new();
        client.connect().await?;
        Ok(client)
    }

    /// Connect to the daemon
    pub async fn connect(&mut self) -> Result<()> {
        *self.state.write().await = ClientState::Connecting;

        #[cfg(unix)]
        let stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| anyhow!("Failed to connect to daemon: {}", e))?;

        #[cfg(windows)]
        let stream = {
            // TODO: 实现Windows Named Pipe连接
            return Err(anyhow!("Windows Named Pipe support not implemented yet"));
        };

        *self.state.write().await = ClientState::Connected;

        // Create message sender and receiver channels
        let (message_sender, message_receiver) = mpsc::unbounded_channel();
        self.message_sender = Some(message_sender);

        let state = self.state.clone();
        let app_state = self.app_state.clone();
        let pending_requests = self.pending_requests.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::handle_connection(
                stream,
                state,
                app_state,
                pending_requests,
                message_receiver,
            )
            .await
            {
                log::error!("Connection handler error: {e}");
            }
        });

        // Perform handshake
        self.perform_handshake().await?;

        // Subscribe to state updates
        self.subscribe_to_events().await?;

        Ok(())
    }

    pub async fn get_state(&self) -> ClientState {
        self.state.read().await.clone()
    }

    pub async fn get_app_state(&self) -> Option<AppState> {
        self.app_state.read().await.clone()
    }

    pub(crate) fn get_app_state_ref(&self) -> Arc<RwLock<Option<AppState>>> {
        self.app_state.clone()
    }

    pub async fn send_rpc_request(
        &self,
        service: crate::ipc::protocol::RpcService,
    ) -> Result<serde_json::Value> {
        let envelope = IpcEnvelope::new(IpcKind::Request(service), serde_json::Value::Null);
        let request_uuid = envelope.uuid.clone();
        log::debug!("Sending RPC request id : {request_uuid}");

        let (response_sender, response_receiver) = oneshot::channel();

        // add pending request
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(request_uuid.clone(), response_sender);
        }

        if let Some(sender) = &self.message_sender {
            sender.send(envelope)?;
            const TIMEOUT_SEC: u64 = 60 * 60 * 24;

            // wait for response with timeout
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(TIMEOUT_SEC),
                response_receiver,
            )
            .await
            {
                Ok(Ok(response)) => Ok(response),
                Ok(Err(_)) => {
                    // clean pending request
                    self.pending_requests.write().await.remove(&request_uuid);
                    Err(anyhow!("Response channel closed"))
                }
                Err(_) => {
                    // timeout and clean pending request
                    self.pending_requests.write().await.remove(&request_uuid);
                    Err(anyhow!("Request timeout"))
                }
            }
        } else {
            // connect error and clean pending request
            self.pending_requests.write().await.remove(&request_uuid);
            Err(anyhow!("Not connected to daemon"))
        }
    }

    async fn perform_handshake(&self) -> Result<()> {
        let hello_msg = HelloMessage {
            version: 1,
            client_info: Some("dball-tui".to_owned()),
            server_name: None,
            supported_features: vec!["basic_rpc".to_owned(), "state_subscription".to_owned()],
        };

        let envelope = IpcEnvelope::new(IpcKind::Hello, serde_json::to_value(hello_msg)?);

        if let Some(sender) = &self.message_sender {
            sender.send(envelope)?;
            *self.state.write().await = ClientState::Authenticated;
            Ok(())
        } else {
            Err(anyhow!("Not connected"))
        }
    }

    async fn subscribe_to_events(&self) -> Result<()> {
        let subscribe_msg = SubscribeMessage {
            events: vec![
                EventType::AppStateChange,
                EventType::TicketUpdate,
                EventType::SpotUpdate,
                EventType::SystemHealth,
                EventType::ApiStatus,
            ],
            filter: None,
        };

        let envelope = IpcEnvelope::new(IpcKind::Subscribe, serde_json::to_value(subscribe_msg)?);

        if let Some(sender) = &self.message_sender {
            sender.send(envelope)?;
            *self.state.write().await = ClientState::Subscribed;
            Ok(())
        } else {
            Err(anyhow!("Not connected"))
        }
    }

    async fn handle_connection(
        mut stream: UnixStream,
        state: Arc<RwLock<ClientState>>,
        app_state: Arc<RwLock<Option<AppState>>>,
        pending_requests: Arc<RwLock<HashMap<String, oneshot::Sender<serde_json::Value>>>>,
        mut message_receiver: mpsc::UnboundedReceiver<IpcEnvelope>,
    ) -> Result<()> {
        let mut buffer = FrameBuffer::new();
        let mut read_buf = vec![0u8; 4096];

        loop {
            tokio::select! {
                result = stream.read(&mut read_buf) => {
                    match result {
                        Ok(0) => {
                            log::error!("Server disconnected");
                            *state.write().await = ClientState::Disconnected;
                            break;
                        }
                        Ok(n) => {
                            buffer.push(&read_buf[0..n]);

                            while let Some(envelope) = buffer.try_decode::<serde_json::Value>()? {
                                Self::process_server_message(envelope, &app_state, &pending_requests).await?;
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to read from server: {e}");
                            *state.write().await = ClientState::Error(e.to_string());
                            break;
                        }
                    }
                }

                Some(envelope) = message_receiver.recv() => {
                    if let Err(e) = Self::send_message(&mut stream, &envelope).await {
                        log::error!("Failed to send message: {e}");
                        *state.write().await = ClientState::Error(e.to_string());
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_server_message(
        envelope: IpcEnvelope,
        app_state: &Arc<RwLock<Option<AppState>>>,
        pending_requests: &Arc<RwLock<HashMap<String, oneshot::Sender<serde_json::Value>>>>,
    ) -> Result<()> {
        match envelope.kind {
            IpcKind::Hello => {
                log::info!("Received Hello response from server");
            }
            IpcKind::Response => {
                let mut pending = pending_requests.write().await;
                if let Some(sender) = pending.remove(&envelope.uuid) {
                    // parse ResponseMessage
                    if sender.send(envelope.msg).is_err() {
                        log::error!("Failed to send response for UUID: {}", envelope.uuid);
                    }
                } else {
                    log::warn!("No pending request found for UUID: {}", envelope.uuid);
                    return Ok(());
                };
            }
            IpcKind::Event => {
                if let Ok(state) = serde_json::from_value::<AppState>(envelope.msg) {
                    *app_state.write().await = Some(state);
                    log::debug!("Updated app state from event");
                }
            }
            IpcKind::Err => {
                log::error!("Received error from server: {:?}", envelope.msg);
            }
            _ => {
                log::warn!("Unexpected message from server: {:?}", envelope.kind);
            }
        }

        Ok(())
    }

    async fn send_message(stream: &mut UnixStream, envelope: &IpcEnvelope) -> Result<()> {
        let encoded = IpcCodec::encode(envelope)?;
        stream.write_all(&encoded).await?;
        Ok(())
    }
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ipc_client_creation() {
        let client = IpcClient::new();
        let state = client.get_state().await;
        assert!(matches!(state, ClientState::Disconnected));
    }

    #[test]
    fn test_client_state_debug() {
        let state = ClientState::Connected;
        let debug_str = format!("{state:?}");
        assert!(debug_str.contains("Connected"));
    }
}
