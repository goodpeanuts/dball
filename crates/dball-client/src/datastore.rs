use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use std::env;

pub mod models;
pub mod schema;

pub fn establish_connection() -> anyhow::Result<SqliteConnection> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("Connecting to database at: {database_url}");
    SqliteConnection::establish(&database_url)
        .map_err(|e| anyhow::anyhow!("Error connecting to {}", e))
}

pub fn get_connection() -> anyhow::Result<SqliteConnection> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .map_err(|e| anyhow::anyhow!("Error connecting to {}", e))
}
