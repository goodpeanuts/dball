use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

use super::{InstanceLock, IpcServer};
use crate::ipc::protocol::AppState;

/// daemon process main service
///
/// integrate existing service modules to manage the daemon process' lifecycle
pub struct DaemonService {
    /// application state
    state: Arc<RwLock<AppState>>,
    /// state broadcaster
    state_broadcaster: broadcast::Sender<AppState>,
    /// IPC server
    ipc_server: Option<IpcServer>,
    /// instance lock
    _instance_lock: InstanceLock,
    /// service running flag
    running: Arc<RwLock<bool>>,
}

impl DaemonService {
    pub async fn new() -> Result<Self> {
        let instance_lock = InstanceLock::acquire().await?;

        let initial_state = Self::create_initial_state().await?;
        let state = Arc::new(RwLock::new(initial_state.clone()));

        let (state_broadcaster, _) = broadcast::channel(100);

        let service = Self {
            state,
            state_broadcaster,
            ipc_server: None,
            _instance_lock: instance_lock,
            running: Arc::new(RwLock::new(false)),
        };

        if let Err(e) = service.state_broadcaster.send(initial_state) {
            log::warn!("Failed to send initial state: {e}");
        }

        Ok(service)
    }

    /// start daemon service
    pub async fn start(&mut self) -> Result<()> {
        log::info!("Starting daemon service...");

        // set running flag
        *self.running.write().await = true;

        // start IPC server
        let ipc_server = IpcServer::new(self.state.clone(), self.state_broadcaster.clone()).await?;

        self.ipc_server = Some(ipc_server);

        log::info!("Daemon service started successfully");
        Ok(())
    }

    /// run daemon service main loop
    pub async fn run(&self) -> Result<()> {
        log::info!("Daemon service is running");

        // set signal handler
        let running = self.running.clone();
        let running_clone = running.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::handle_signals(running_clone).await {
                log::error!("Signal handler error: {e}");
            }
        });

        // start IPC server
        if let Some(ref ipc_server) = self.ipc_server {
            let server_handle = ipc_server.start().await?;

            // wait until stop signal
            while *running.read().await {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            // stop IPC server
            server_handle.abort();
        }

        log::info!("Daemon service stopped");
        Ok(())
    }

    /// stop daemon service
    pub async fn shutdown(&self) -> Result<()> {
        log::info!("Shutting down daemon service...");

        // set stop flag
        *self.running.write().await = false;

        // IPC server will stop in main loop

        log::info!("Daemon service shutdown completed");
        Ok(())
    }

    /// subscribe state changes
    pub fn subscribe_state(&self) -> broadcast::Receiver<AppState> {
        self.state_broadcaster.subscribe()
    }

    /// update application state
    pub async fn update_state<F>(&self, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut AppState),
    {
        let mut state = self.state.write().await;
        update_fn(&mut state);

        // broadcast state update
        if let Err(e) = self.state_broadcaster.send(state.clone()) {
            log::warn!("Failed to broadcast state update: {e}");
        }

        Ok(())
    }

    /// get current application state
    pub async fn get_state(&self) -> AppState {
        self.state.read().await.clone()
    }

    // TODO: remove this method once IPC server is fully implemented
    /// create initial application state
    async fn create_initial_state() -> Result<AppState> {
        use crate::db::{spot, tickets};
        use crate::ipc::protocol::{ApiStatusInfo, GenerationStatus};
        use chrono::Utc;
        use std::time::Duration;

        // get latest ticket information
        let (current_period, next_period) = match crate::service::update_latest_ticket().await {
            Ok(latest_period) => {
                let next = crate::service::get_next_period().await.unwrap_or_else(|_| {
                    let next_num = latest_period.period.parse::<i32>().unwrap_or(25001) + 1;
                    next_num.to_string()
                });
                (latest_period.period, next)
            }
            Err(e) => {
                log::warn!("Failed to get latest period: {e}, using defaults");
                ("25001".to_owned(), "25002".to_owned())
            }
        };

        // get latest ticket information
        let latest_ticket = tickets::get_latest_tickets(1)
            .ok()
            .and_then(|mut tickets| tickets.pop())
            .and_then(|ticket| ticket.to_dball().ok());

        // get unopened ticket count
        let unprize_spots_count = spot::get_all_unprize_spots()
            .map(|spots| spots.len() as u32)
            .unwrap_or(0);

        // calculate total investment and return
        let (total_investment, total_return) = spot::get_all_spots()
            .map(|spots| {
                spots.iter().fold((0.0, 0.0), |(inv, ret), spot| {
                    let investment = 2.0; // 每注2元
                    let return_amount = spot
                        .prize_status
                        .map(|status| match status {
                            1 => 10000000.0,
                            2 => 200000.0,
                            3 => 3000.0,
                            4 => 200.0,
                            5 => 10.0,
                            6 => 5.0,
                            _ => 0.0,
                        })
                        .unwrap_or(0.0);
                    (inv + investment, ret + return_amount)
                })
            })
            .unwrap_or((0.0, 0.0));

        Ok(AppState {
            current_period,
            next_period,
            last_draw_time: tickets::get_latest_tickets(1)
                .ok()
                .and_then(|mut tickets| tickets.pop())
                .map(|ticket| ticket.time.and_utc()),
            next_draw_time: None,
            latest_ticket,
            pending_tickets: vec![],
            unprize_spots_count,
            total_investment,
            total_return,
            api_status: ApiStatusInfo {
                api_provider: "mxnzp".to_owned(),
                last_success: None,
                success_rate: 0.0,
                average_response_time: Duration::from_millis(1000),
            },
            last_update: Utc::now(),
            daemon_uptime: Duration::from_secs(0),
            generation_status: GenerationStatus::Idle,
            last_generation_time: None,
        })
    }

    /// handle signals for graceful shutdown and configuration reload
    async fn handle_signals(running: Arc<RwLock<bool>>) -> Result<()> {
        #[cfg(unix)]
        {
            use tokio::signal;

            let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;
            let mut sigusr1 = signal::unix::signal(signal::unix::SignalKind::user_defined1())?;

            loop {
                tokio::select! {
                    _ = signal::ctrl_c() => {
                        log::info!("Received SIGINT, shutting down...");
                        *running.write().await = false;
                        break;
                    }

                    _ = sigterm.recv() => {
                        log::info!("Received SIGTERM, shutting down...");
                        *running.write().await = false;
                        break;
                    }

                    _ = sigusr1.recv() => {
                        log::info!("Received SIGUSR1, reloading configuration...");
                        // TODO: 实现配置重载逻辑
                    }
                }
            }
        }

        #[cfg(not(unix))]
        {
            use tokio::signal;

            signal::ctrl_c().await?;
            log::info!("Received SIGINT, shutting down...");
            *running.write().await = false;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_daemon_service_creation() {
        // 注意: 这个测试可能会因为实例锁而失败，如果已有守护进程在运行
        // 在实际测试中，可能需要使用不同的锁文件路径

        // 暂时跳过，因为需要处理实例锁的测试
        // let service = DaemonService::new().await;
        // assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_state_subscription() {
        // 创建一个基本的状态用于测试
        let initial_state = AppState {
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

        let _state = Arc::new(RwLock::new(initial_state.clone()));
        let (broadcaster, mut receiver) = broadcast::channel(10);

        // 发送状态
        broadcaster.send(initial_state.clone()).unwrap();

        // 接收状态
        let received_state = receiver.recv().await.unwrap();
        assert_eq!(received_state.current_period, "test");
    }
}
