use chrono::{DateTime, Utc};
use dball_combora::dball::DBall;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Rpc service definition
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RpcService {
    GenerateBatchSpots,

    UpdateAllUnprizeSpots,
    DeprecatedLastBatchUnprizedSpot,

    UpdateLatestTicket,
    CrawlAllTickets,
    UpdateTicketsByPeriod(Vec<String>),
    UpdateTicketsWithYear(i32),

    GetCurrentState,
    GetLatestPeriod,
    GetUnprizeSpots,
    GetPrizedSpots,

    Shutdown,
    Restart,
}

/// 握手消息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelloMessage {
    /// version number
    pub version: u16,

    /// C2D
    pub client_info: Option<String>,

    /// D2C
    pub server_name: Option<String>,

    /// Supported features
    pub supported_features: Vec<String>,
}

/// 订阅消息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscribeMessage {
    /// subscribe events
    pub events: Vec<EventType>,
    /// optional event filter
    pub filter: Option<String>,
}

/// 事件类型
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    /// app state change
    AppStateChange,
    /// ticket update
    TicketUpdate,
    /// spot update
    SpotUpdate,
    /// system health
    SystemHealth,
    /// api status
    ApiStatus,
}

// /// Response message
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct ResponseMessage {
//     /// UUID of the corresponding request
//     pub request_uuid: String,
//     /// Whether the operation was successful
//     pub success: bool,
//     /// Return data (JSON format)
//     pub data: Option<serde_json::Value>,
//     /// Error information
//     pub error: Option<String>,
// }

/// Event notification message
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventMessage {
    pub event_type: EventType,

    pub data: serde_json::Value,

    pub source: String,
}

/// Error message
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorMessage {
    /// Error code
    pub code: u32,
    /// Error description
    pub message: String,
    /// Detailed error information
    pub details: Option<String>,
}

/// 应用状态
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppState {
    pub current_period: String,

    pub next_period: String,

    pub last_draw_time: Option<DateTime<Utc>>,

    pub next_draw_time: Option<DateTime<Utc>>,

    pub latest_ticket: Option<DBall>,

    pub pending_tickets: Vec<String>,

    pub unprize_spots_count: u32,

    pub total_investment: f64,

    pub total_return: f64,

    pub api_status: ApiStatusInfo,

    pub last_update: DateTime<Utc>,

    pub daemon_uptime: Duration,

    pub generation_status: GenerationStatus,

    pub last_generation_time: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TicketInfo {
    pub period: String,
    pub red_balls: Vec<u8>,
    pub blue_ball: u8,
    pub draw_time: DateTime<Utc>,
}

/// API状态信息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiStatusInfo {
    pub api_provider: String,
    pub last_success: Option<DateTime<Utc>>,
    pub success_rate: f64,
    pub average_response_time: Duration,
}

/// 生成状态
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GenerationStatus {
    Idle,
    Generating,
    Generated,
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_service_serialization() {
        let service =
            RpcService::UpdateTicketsByPeriod(vec!["2024001".to_string(), "2024002".to_string()]);
        let serialized = serde_json::to_string(&service).expect("Failed to serialize");
        let deserialized: RpcService =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        match deserialized {
            RpcService::UpdateTicketsByPeriod(periods) => {
                assert_eq!(periods.len(), 2);
                assert_eq!(periods[0], "2024001");
            }
            _ => panic!("Wrong variant deserialized"),
        }
    }

    #[test]
    fn test_hello_message() {
        let hello = HelloMessage {
            version: 1,
            client_info: Some("test_client".to_string()),
            server_name: None,
            supported_features: vec!["basic".to_string(), "advanced".to_string()],
        };

        let serialized = serde_json::to_string(&hello).expect("Failed to serialize");
        let deserialized: HelloMessage =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(hello.version, deserialized.version);
        assert_eq!(hello.client_info, deserialized.client_info);
        assert_eq!(
            hello.supported_features.len(),
            deserialized.supported_features.len()
        );
    }

    #[test]
    fn test_app_state_creation() {
        let app_state = AppState {
            current_period: "2024001".to_string(),
            next_period: "2024002".to_string(),
            last_draw_time: Some(Utc::now()),
            next_draw_time: None,
            latest_ticket: None,
            pending_tickets: vec![],
            unprize_spots_count: 0,
            total_investment: 0.0,
            total_return: 0.0,
            api_status: ApiStatusInfo {
                api_provider: "mxnzp".to_string(),
                last_success: None,
                success_rate: 0.95,
                average_response_time: Duration::from_millis(500),
            },
            last_update: Utc::now(),
            daemon_uptime: Duration::from_secs(3600),
            generation_status: GenerationStatus::Idle,
            last_generation_time: None,
        };

        // 确保可以序列化
        let serialized = serde_json::to_string(&app_state).expect("Failed to serialize AppState");
        assert!(!serialized.is_empty());
    }
}
