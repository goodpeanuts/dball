use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;
use std::sync::LazyLock;

pub mod spot;
pub mod ticket_log;
pub mod tickets;

#[derive(Debug)]
struct SqliteConnectionCustomizer;

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for SqliteConnectionCustomizer {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        use diesel::RunQueryDsl as _;

        // using WAL mode for better concurrency
        diesel::sql_query("PRAGMA journal_mode = WAL;")
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        // ! may lost last transaction on crash
        diesel::sql_query("PRAGMA synchronous = NORMAL;")
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        // ! may lost last transaction on crash
        diesel::sql_query("PRAGMA busy_timeout = 30000;")
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        // foreign key constraints
        diesel::sql_query("PRAGMA foreign_keys = ON;")
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;

        Ok(())
    }
}

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
        .max_size(10)
        .connection_timeout(std::time::Duration::from_secs(30))
        .connection_customizer(Box::new(SqliteConnectionCustomizer))
        .build(manager)
        .expect("Failed to create pool")
});

pub fn establish_db_connection() -> anyhow::Result<SqliteConnection> {
    let database_url = get_database_url();
    let mut conn = SqliteConnection::establish(&database_url).map_err(|e| {
        let err_message = format!("Error connecting to {database_url}: {e}");
        log::error!("{err_message}");
        anyhow::anyhow!("{err_message}")
    })?;

    let customizer = SqliteConnectionCustomizer;
    customizer
        .on_acquire(&mut conn)
        .map_err(|e| anyhow::anyhow!("Failed to customize connection: {:?}", e))?;

    Ok(conn)
}

fn get_db_connection() -> anyhow::Result<PooledConnection<ConnectionManager<SqliteConnection>>> {
    DB_POOL
        .get()
        .map_err(|e| anyhow::anyhow!("Failed to get DB connection: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn get_test_db_connection() {
        log::info!("Starting database connection test");
        assert!(get_db_connection().is_ok());
    }
}
