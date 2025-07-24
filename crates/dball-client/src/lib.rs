//! 数据库客户端模块
//!
//! 负责处理所有的数据库相关操作，包括数据的存储、查询和管理。

pub mod datastore;
pub mod service;

// 重新导出常用的类型和函数
pub use datastore::models::{NewTicketLog, TicketLog};
pub use datastore::{establish_connection, get_connection};
pub use service::*;
