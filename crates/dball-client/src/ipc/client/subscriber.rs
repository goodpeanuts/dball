use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, watch};

use super::IpcClient;
use crate::ipc::protocol::{AppState, EventType};

/// manages state subscription and updates
#[derive(Clone)]
pub struct StateSubscriber {
    current_state: Arc<RwLock<Option<AppState>>>,
    state_sender: watch::Sender<Option<AppState>>,
    state_receiver: watch::Receiver<Option<AppState>>,
    event_filter: Vec<EventType>,
}

impl StateSubscriber {
    pub fn new() -> Self {
        let (state_sender, state_receiver) = watch::channel(None);

        Self {
            current_state: Arc::new(RwLock::new(None)),
            state_sender,
            state_receiver,
            event_filter: vec![
                EventType::AppStateChange,
                EventType::TicketUpdate,
                EventType::SpotUpdate,
                EventType::SystemHealth,
                EventType::ApiStatus,
            ],
        }
    }

    pub fn with_event_filter(mut self, filter: Vec<EventType>) -> Self {
        self.event_filter = filter;
        self
    }

    pub async fn start_subscription(&self, client: &mut IpcClient) -> Result<()> {
        log::info!("Starting state subscription");

        if let Some(initial_state) = client.get_app_state().await {
            self.update_state(initial_state).await?;
        }

        let client_state = client.get_app_state_ref();
        let current_state = self.current_state.clone();
        let state_sender = self.state_sender.clone();

        tokio::spawn(async move {
            let mut last_update: Option<AppState> = None;

            #[expect(clippy::infinite_loop)]
            loop {
                // check if the client state has been updated
                let client_app_state = client_state.read().await.clone();

                // compare the state to see if it has changed
                let state_changed = match (&last_update, &client_app_state) {
                    (None, Some(_)) | (Some(_), None) => true,
                    (Some(last), Some(current)) => last.last_update != current.last_update,
                    (None, None) => false,
                };

                if state_changed {
                    if let Some(new_state) = client_app_state.clone() {
                        // update current state
                        *current_state.write().await = Some(new_state.clone());

                        // notify subscribers
                        if let Err(e) = state_sender.send(Some(new_state.clone())) {
                            log::error!("Failed to send state update: {e}");
                        }

                        last_update = Some(new_state);
                        log::debug!("State updated from daemon");
                    } else {
                        // update current state
                        *current_state.write().await = None;
                        if let Err(e) = state_sender.send(None) {
                            log::error!("Failed to send state clear: {e}");
                        }
                        last_update = None;
                        log::debug!("State cleared");
                    }
                }

                // sleep for a short duration to avoid busy-waiting
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });

        log::info!("State subscription started");
        Ok(())
    }

    /// get current state
    pub async fn get_current_state(&self) -> Option<AppState> {
        self.current_state.read().await.clone()
    }

    pub fn subscribe_to_changes(&self) -> watch::Receiver<Option<AppState>> {
        self.state_receiver.clone()
    }

    pub async fn wait_for_change(&mut self) -> Result<Option<AppState>> {
        self.state_receiver.changed().await?;
        Ok(self.state_receiver.borrow().clone())
    }

    pub async fn wait_for_condition<F>(&mut self, condition: F) -> Result<AppState>
    where
        F: Fn(&AppState) -> bool,
    {
        loop {
            // check if the current state satisfies the condition
            if let Some(current) = self.get_current_state().await {
                if condition(&current) {
                    return Ok(current);
                }
            }

            // wait for the next change
            self.state_receiver.changed().await?;

            if let Some(new_state) = &*self.state_receiver.borrow() {
                if condition(new_state) {
                    return Ok(new_state.clone());
                }
            }
        }
    }

    async fn update_state(&self, new_state: AppState) -> Result<()> {
        *self.current_state.write().await = Some(new_state.clone());

        // notify subscribers
        self.state_sender.send(Some(new_state))?;

        Ok(())
    }

    /// clear current state
    pub async fn clear_state(&self) -> Result<()> {
        *self.current_state.write().await = None;
        self.state_sender.send(None)?;
        Ok(())
    }

    /// Subscribe to events from the daemon
    pub async fn get_state_stats(&self) -> StateStats {
        let state = self.current_state.read().await;

        match &*state {
            Some(app_state) => StateStats {
                has_state: true,
                last_update: Some(app_state.last_update),
                daemon_uptime: Some(app_state.daemon_uptime),
                unprize_spots_count: app_state.unprize_spots_count,
                api_provider: Some(app_state.api_status.api_provider.clone()),
                connection_status: ConnectionStatus::Connected,
            },
            None => StateStats {
                has_state: false,
                last_update: None,
                daemon_uptime: None,
                unprize_spots_count: 0,
                api_provider: None,
                connection_status: ConnectionStatus::Disconnected,
            },
        }
    }
}

impl Default for StateSubscriber {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct StateStats {
    pub has_state: bool,

    pub last_update: Option<chrono::DateTime<chrono::Utc>>,

    pub daemon_uptime: Option<std::time::Duration>,

    pub unprize_spots_count: u32,

    pub api_provider: Option<String>,

    pub connection_status: ConnectionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Reconnecting,
    Error(String),
}

#[derive(Debug, Clone)]
pub enum StateEvent {
    Updated(Box<AppState>),
    Cleared,
    ConnectionChanged(ConnectionStatus),
}

pub struct StateEventStream {
    receiver: mpsc::UnboundedReceiver<StateEvent>,
}

impl StateEventStream {
    /// wait for the next state event
    pub async fn next(&mut self) -> Option<StateEvent> {
        self.receiver.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_state_subscriber_creation() {
        let subscriber = StateSubscriber::new();
        let state = subscriber.get_current_state().await;
        assert!(state.is_none());
    }

    #[tokio::test]
    async fn test_state_subscription() {
        let subscriber = StateSubscriber::new();
        let mut receiver = subscriber.subscribe_to_changes();

        // 初始状态应该是None
        assert!(receiver.borrow().is_none());

        // 创建测试状态
        let test_state = AppState {
            current_period: "test".to_string(),
            next_period: "test".to_string(),
            last_draw_time: None,
            next_draw_time: None,
            latest_ticket: None,
            pending_tickets: vec![],
            unprize_spots_count: 0,
            total_investment: 0.0,
            total_return: 0.0,
            api_status: crate::ipc::protocol::ApiStatusInfo {
                api_provider: "test".to_string(),
                last_success: None,
                success_rate: 0.0,
                average_response_time: Duration::from_millis(1000),
            },
            last_update: chrono::Utc::now(),
            daemon_uptime: Duration::from_secs(0),
            generation_status: crate::ipc::protocol::GenerationStatus::Idle,
            last_generation_time: None,
        };

        // 更新状态
        subscriber.update_state(test_state.clone()).await.unwrap();

        // 等待状态变更通知
        receiver.changed().await.unwrap();

        // 验证状态已更新
        assert!(receiver.borrow().is_some());
        let updated_state = subscriber.get_current_state().await.unwrap();
        assert_eq!(updated_state.current_period, "test");
    }

    #[tokio::test]
    async fn test_state_stats() {
        let subscriber = StateSubscriber::new();

        // 初始统计
        let stats = subscriber.get_state_stats().await;
        assert!(!stats.has_state);
        assert_eq!(stats.connection_status, ConnectionStatus::Disconnected);

        // 添加状态后的统计
        let test_state = AppState {
            current_period: "test".to_string(),
            next_period: "test".to_string(),
            last_draw_time: None,
            next_draw_time: None,
            latest_ticket: None,
            pending_tickets: vec![],
            unprize_spots_count: 5,
            total_investment: 0.0,
            total_return: 0.0,
            api_status: crate::ipc::protocol::ApiStatusInfo {
                api_provider: "mxnzp".to_string(),
                last_success: None,
                success_rate: 0.0,
                average_response_time: Duration::from_millis(1000),
            },
            last_update: chrono::Utc::now(),
            daemon_uptime: Duration::from_secs(0),
            generation_status: crate::ipc::protocol::GenerationStatus::Idle,
            last_generation_time: None,
        };

        subscriber.update_state(test_state).await.unwrap();

        let stats = subscriber.get_state_stats().await;
        assert!(stats.has_state);
        assert_eq!(stats.unprize_spots_count, 5);
        assert_eq!(stats.api_provider, Some("mxnzp".to_string()));
    }

    #[tokio::test]
    async fn test_wait_for_condition() {
        let subscriber = StateSubscriber::new();
        let subscriber_clone = subscriber.clone();

        // 在后台任务中更新状态
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;

            let test_state = AppState {
                current_period: "target".to_string(),
                next_period: "test".to_string(),
                last_draw_time: None,
                next_draw_time: None,
                latest_ticket: None,
                pending_tickets: vec![],
                unprize_spots_count: 0,
                total_investment: 0.0,
                total_return: 0.0,
                api_status: crate::ipc::protocol::ApiStatusInfo {
                    api_provider: "test".to_string(),
                    last_success: None,
                    success_rate: 0.0,
                    average_response_time: Duration::from_millis(1000),
                },
                last_update: chrono::Utc::now(),
                daemon_uptime: Duration::from_secs(0),
                generation_status: crate::ipc::protocol::GenerationStatus::Idle,
                last_generation_time: None,
            };

            subscriber_clone.update_state(test_state).await.unwrap();
        });

        // 等待特定条件
        let mut receiver = subscriber.subscribe_to_changes();
        let result = tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                receiver.changed().await.unwrap();
                if let Some(state) = &*receiver.borrow() {
                    if state.current_period == "target" {
                        return state.clone();
                    }
                }
            }
        })
        .await;

        assert!(result.is_ok());
        let state = result.unwrap();
        assert_eq!(state.current_period, "target");
    }
}
