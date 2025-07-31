use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::protocol::RpcService;

/// IPC message envelope format
/// All IPC messages are encapsulated using this format,
/// which includes protocol version, message ID, type, and specific content
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IpcEnvelope<T> {
    /// Protocol version, currently 1
    pub proto: u16,
    /// Message unique identifier
    pub uuid: String,
    /// Basic communication type
    pub kind: IpcKind,
    /// Specific message content
    pub msg: T,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
}

impl<T> IpcEnvelope<T> {
    pub fn new(kind: IpcKind, msg: T) -> Self {
        Self {
            proto: 1,
            uuid: Uuid::new_v4().to_string(),
            kind,
            msg,
            timestamp: Utc::now(),
        }
    }

    /// Create a new IPC message envelope with a specific UUID
    pub fn new_with_uuid(kind: IpcKind, msg: T, uuid: String) -> Self {
        Self {
            proto: 1,
            uuid,
            kind,
            msg,
            timestamp: Utc::now(),
        }
    }
}

/// IPC basic communication types
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum IpcKind {
    /// Hello message, used for protocol handshake
    Hello,
    /// Subscription request
    Subscribe,
    /// Client RPC request
    Request(RpcService),
    /// Server response (success/failure)
    Response(bool),
    /// Event notification (status change)
    Event,
    /// Error message
    Err,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::protocol::HelloMessage;

    #[test]
    fn test_envelope_creation() {
        let hello_msg = HelloMessage {
            version: 1,
            client_info: Some("test_client".to_string()),
            server_name: None,
            supported_features: vec!["basic".to_string()],
        };

        let envelope = IpcEnvelope::new(IpcKind::Hello, hello_msg);

        assert_eq!(envelope.proto, 1);
        assert!(!envelope.uuid.is_empty());
        assert!(matches!(envelope.kind, IpcKind::Hello));
    }

    #[test]
    fn test_envelope_serialization() {
        let hello_msg = HelloMessage {
            version: 1,
            client_info: Some("test_client".to_string()),
            server_name: None,
            supported_features: vec!["basic".to_string()],
        };

        let envelope = IpcEnvelope::new(IpcKind::Hello, hello_msg);
        let serialized = serde_json::to_string(&envelope).expect("Failed to serialize");
        let deserialized: IpcEnvelope<HelloMessage> =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(envelope.uuid, deserialized.uuid);
        assert_eq!(envelope.proto, deserialized.proto);
    }
}
