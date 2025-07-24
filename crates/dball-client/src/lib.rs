//! 数据库客户端模块
//!
//! 负责处理所有的数据库相关操作，包括数据的存储、查询和管理。

use std::{path::PathBuf, sync::LazyLock};

pub(crate) static ENV_GUARD: LazyLock<Result<PathBuf, anyhow::Error>> = LazyLock::new(|| {
    dotenvy::dotenv().map_err(|e| anyhow::anyhow!("Failed to load .env file: {e}"))
});

/// allow env file not found
pub(crate) fn init_env_unnecessarily() {
    if let Err(e) = crate::ENV_GUARD.as_ref() {
        log::error!("{e}");
    }
}

/// load env file, panic if failed
pub(crate) fn init_env() {
    crate::ENV_GUARD
        .as_ref()
        .expect("Failed to load environment variables. Ensure .env file exists and is correctly configured.");
}

pub(crate) fn parse_from_env<T: std::str::FromStr>(key: &str) -> T
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    std::env::var(key)
        .unwrap_or_else(|_| panic!("{key} must be set"))
        .parse::<T>()
        .unwrap_or_else(|e| panic!("Failed to parse {key}: {e}"))
}

pub mod db;
pub mod models;
pub mod request;
pub mod service;
