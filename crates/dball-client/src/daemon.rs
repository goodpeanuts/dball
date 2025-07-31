//! 守护进程模块
//!
//! 提供守护进程的核心功能，包括服务管理、IPC服务器、状态管理等

pub mod ipc_server;
pub mod lock;
pub mod service;

// 重新导出主要类型
pub use ipc_server::IpcServer;
pub use lock::InstanceLock;
pub use service::DaemonService;
