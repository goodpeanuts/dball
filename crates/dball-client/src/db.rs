use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;
use std::env;
use std::sync::LazyLock;

use crate::init_env_unnecessarily;

static DB_POOL: LazyLock<Pool<ConnectionManager<SqliteConnection>>> = LazyLock::new(|| {
    init_env_unnecessarily();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder()
        .max_size(4) // 根据你的并发量调整
        .build(manager)
        .expect("Failed to create pool")
});

pub fn establish_connection() -> anyhow::Result<SqliteConnection> {
    init_env_unnecessarily();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url).map_err(|e| {
        let err_message = format!("Error connecting to {database_url}: {e}");
        log::error!("{err_message}");
        anyhow::anyhow!("{err_message}")
    })
}

pub fn get_db_connection() -> anyhow::Result<PooledConnection<ConnectionManager<SqliteConnection>>>
{
    DB_POOL
        .get()
        .map_err(|e| anyhow::anyhow!("Failed to get DB connection: {}", e))
}
