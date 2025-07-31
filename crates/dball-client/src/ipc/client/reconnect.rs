use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

use super::IpcClient;

pub struct ReconnectManager {
    min_interval: Duration,

    max_interval: Duration,

    /// Exponential backoff multiplier
    backoff_multiplier: f64,

    /// None means unlimited retries
    max_attempts: Option<u32>,
}

impl ReconnectManager {
    pub fn new() -> Self {
        Self {
            min_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            max_attempts: None, // unlimited retries
        }
    }

    pub fn min_interval(mut self, interval: Duration) -> Self {
        self.min_interval = interval;
        self
    }

    pub fn max_interval(mut self, interval: Duration) -> Self {
        self.max_interval = interval;
        self
    }

    pub fn backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    pub fn max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = Some(attempts);
        self
    }

    pub async fn reconnect_loop<F, Fut>(&self, mut connect_fn: F) -> Result<()>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let mut attempt = 0;
        let mut current_interval = self.min_interval;

        loop {
            attempt += 1;

            if let Some(max) = self.max_attempts {
                if attempt > max {
                    return Err(anyhow::anyhow!("Max reconnect attempts ({}) exceeded", max));
                }
            }

            log::info!("Reconnect attempt #{attempt}");

            match connect_fn().await {
                Ok(_) => {
                    log::info!("Reconnected successfully after {attempt} attempts");
                    return Ok(());
                }
                Err(e) => {
                    log::warn!("Reconnect attempt #{attempt} failed: {e}");

                    log::debug!("Waiting {current_interval:?} before next attempt");
                    sleep(current_interval).await;

                    current_interval = std::cmp::min(
                        Duration::from_millis(
                            (current_interval.as_millis() as f64 * self.backoff_multiplier) as u64,
                        ),
                        self.max_interval,
                    );
                }
            }
        }
    }

    pub async fn monitor_and_reconnect(&self, client: Arc<RwLock<IpcClient>>) -> Result<()> {
        log::info!("Starting connection monitor");

        {
            let mut client_guard = client.write().await;
            if let Err(e) = client_guard.connect().await {
                log::error!("Initial connection failed: {e}");

                let client_clone = client.clone();
                self.reconnect_loop(move || {
                    let client = client_clone.clone();
                    async move {
                        let mut client_guard = client.write().await;
                        client_guard.connect().await
                    }
                })
                .await?;
            }
        }

        loop {
            sleep(Duration::from_secs(5)).await;

            let state = {
                let client_guard = client.read().await;
                client_guard.get_state().await
            };

            match state {
                super::client::ClientState::Disconnected | super::client::ClientState::Error(_) => {
                    log::warn!("Connection lost, attempting to reconnect...");

                    let client_clone = client.clone();
                    if let Err(e) = self
                        .reconnect_loop(move || {
                            let client = client_clone.clone();
                            async move {
                                let mut client_guard = client.write().await;
                                client_guard.connect().await
                            }
                        })
                        .await
                    {
                        log::error!("Failed to reconnect: {e}");
                        return Err(e);
                    }
                }
                _ => {
                    log::trace!("Connection status: {state:?}");
                }
            }
        }
    }
}

impl Default for ReconnectManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 重连配置
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    pub enabled: bool,
    pub min_interval: Duration,
    pub max_interval: Duration,
    pub backoff_multiplier: f64,
    pub max_attempts: Option<u32>,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            max_attempts: None,
        }
    }
}

impl ReconnectConfig {
    pub fn create_manager(&self) -> ReconnectManager {
        let mut manager = ReconnectManager::new()
            .min_interval(self.min_interval)
            .max_interval(self.max_interval)
            .backoff_multiplier(self.backoff_multiplier);

        if let Some(max) = self.max_attempts {
            manager = manager.max_attempts(max);
        }

        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_reconnect_manager_creation() {
        let manager = ReconnectManager::new();
        assert_eq!(manager.min_interval, Duration::from_secs(1));
        assert_eq!(manager.max_interval, Duration::from_secs(60));
        assert_eq!(manager.backoff_multiplier, 2.0);
        assert!(manager.max_attempts.is_none());
    }

    #[tokio::test]
    async fn test_reconnect_manager_builder() {
        let manager = ReconnectManager::new()
            .min_interval(Duration::from_millis(500))
            .max_interval(Duration::from_secs(30))
            .backoff_multiplier(1.5)
            .max_attempts(5);

        assert_eq!(manager.min_interval, Duration::from_millis(500));
        assert_eq!(manager.max_interval, Duration::from_secs(30));
        assert_eq!(manager.backoff_multiplier, 1.5);
        assert_eq!(manager.max_attempts, Some(5));
    }

    #[tokio::test]
    async fn test_reconnect_with_max_attempts() {
        let manager = ReconnectManager::new()
            .min_interval(Duration::from_millis(10))
            .max_attempts(3);

        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let result = manager
            .reconnect_loop(|| {
                let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst);
                async move {
                    // 总是失败
                    Err(anyhow::anyhow!(
                        "Simulated connection failure #{}",
                        count + 1
                    ))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_reconnect_success_after_failures() {
        let manager = ReconnectManager::new().min_interval(Duration::from_millis(10));

        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();

        let result = manager
            .reconnect_loop(|| {
                let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst);
                async move {
                    if count < 2 {
                        // 前两次失败
                        Err(anyhow::anyhow!(
                            "Simulated connection failure #{}",
                            count + 1
                        ))
                    } else {
                        // 第三次成功
                        Ok(())
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_reconnect_config() {
        let config = ReconnectConfig::default();
        assert!(config.enabled);
        assert_eq!(config.min_interval, Duration::from_secs(1));

        let manager = config.create_manager();
        assert_eq!(manager.min_interval, Duration::from_secs(1));
    }
}
