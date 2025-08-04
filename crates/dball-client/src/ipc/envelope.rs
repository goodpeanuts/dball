use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use uuid::Uuid;

use super::protocol::RpcService;

/// IPC message envelope format
/// All IPC messages are encapsulated using this format,
/// which includes protocol version, message ID, type, and specific content
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IpcEnvelope {
    /// Protocol version, currently 1
    pub proto: u16,
    /// Message unique identifier
    pub uuid: String,
    /// Basic communication type
    pub kind: IpcKind,
    /// Specific message content
    pub msg: serde_json::Value,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
}

impl IpcEnvelope {
    pub fn new(kind: IpcKind, msg: serde_json::Value) -> Self {
        Self {
            proto: 1,
            uuid: Uuid::new_v4().to_string(),
            kind,
            msg,
            timestamp: Utc::now(),
        }
    }

    /// Create a new IPC message envelope with a specific UUID
    pub fn new_with_uuid(kind: IpcKind, msg: serde_json::Value, uuid: String) -> Self {
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
#[derive(Serialize, Deserialize, Debug, Clone, Display)]
pub enum IpcKind {
    /// Hello message, used for protocol handshake
    Hello,
    /// Subscription request
    Subscribe,
    /// Client RPC request
    Request(RpcService),
    /// Server response (success/failure)
    Response,
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
            client_info: Some("test_client".to_owned()),
            server_name: None,
            supported_features: vec!["basic".to_owned()],
        };

        let envelope = IpcEnvelope::new(
            IpcKind::Hello,
            serde_json::to_value(hello_msg).expect("Failed to serialize"),
        );

        assert_eq!(envelope.proto, 1);
        assert!(!envelope.uuid.is_empty());
        assert!(matches!(envelope.kind, IpcKind::Hello));
    }

    #[test]
    fn test_envelope_serialization() {
        let hello_msg = HelloMessage {
            version: 1,
            client_info: Some("test_client".to_owned()),
            server_name: None,
            supported_features: vec!["basic".to_owned()],
        };

        let envelope = IpcEnvelope::new(
            IpcKind::Hello,
            serde_json::to_value(hello_msg).expect("Failed to serialize"),
        );
        let serialized = serde_json::to_string(&envelope).expect("Failed to serialize");
        let deserialized: IpcEnvelope =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(envelope.uuid, deserialized.uuid);
        assert_eq!(envelope.proto, deserialized.proto);
    }
}
