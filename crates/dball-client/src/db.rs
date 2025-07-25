use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;
use std::sync::LazyLock;

fn get_database_url() -> String {
    #[cfg(not(test))]
    let database_url = { std::env::var("DATABASE_URL").expect("DATABASE_URL must be set") };

    #[cfg(test)]
    let database_url = {
        let url = &crate::TEST_ENV_GUARD.test_db;
        log::debug!("Using TEST_DATABASE_URL for testing {}", url.display());
        url.display().to_string()
    };

    database_url
}

static DB_POOL: LazyLock<Pool<ConnectionManager<SqliteConnection>>> = LazyLock::new(|| {
    let database_url = get_database_url();

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder()
        .max_size(4) // 根据你的并发量调整
        .build(manager)
        .expect("Failed to create pool")
});

pub fn establish_connection() -> anyhow::Result<SqliteConnection> {
    let database_url = get_database_url();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn get_test_db_connection() {
        println!("Starting database connection test");
        assert!(get_db_connection().is_ok());
    }
}
