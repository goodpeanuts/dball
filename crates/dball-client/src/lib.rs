use std::{path::PathBuf, sync::LazyLock};

pub(crate) static ENV_GUARD: LazyLock<Result<PathBuf, anyhow::Error>> = LazyLock::new(|| {
    dotenvy::dotenv().map_err(|e| anyhow::anyhow!("Failed to load .env file: {e}"))
});

pub mod db;
pub mod models;
pub mod request;
pub mod service;

const NEVER_NONE_BY_DATABASE: &str = "Should not be None guaranteed by database";

/// load env file, panic if failed
fn init_env() {
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

#[ctor::ctor]
fn init_test_logger() {
    init_env();

    println!("Initializing test logger");
    env_logger::builder()
        .parse_default_env()
        .is_test(true)
        .try_init();
}

#[cfg(test)]
static TEST_ENV_GUARD: LazyLock<TestEnvGuard> = LazyLock::new(|| TestEnvGuard {
    test_db: copy_test_db(),
});

#[cfg(test)]
#[ctor::ctor]
fn new_test_env_guard() {
    log::debug!("Creating test env guard");
    LazyLock::force(&TEST_ENV_GUARD);
}

#[cfg(test)]
struct TestEnvGuard {
    test_db: PathBuf,
}

#[cfg(test)]
/// copy test database from main database
/// return the path of test database
fn copy_test_db() -> std::path::PathBuf {
    let root_path = crate::ENV_GUARD.as_ref().unwrap().parent().unwrap();
    let test_db_url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    let main_db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let main_db_path = root_path.join(main_db_url);
    let test_db_path = root_path.join(test_db_url);

    // clean old test database
    let files_to_remove = [
        test_db_path.clone(),
        test_db_path.with_extension("db-shm"),
        test_db_path.with_extension("db-wal"),
    ];

    for file in &files_to_remove {
        if file.exists() {
            let _ = std::fs::remove_file(file);
        }
    }

    // copy main database as test database
    std::fs::copy(main_db_path, &test_db_path).unwrap();

    log::debug!("Created test db: {}", test_db_path.display());
    test_db_path
}

#[cfg(test)]
#[test]
#[ignore = "manual test to clean test db"]
fn clean_test_env() {
    let path = &TEST_ENV_GUARD.test_db;
    if let Err(e) = std::fs::remove_file(path) {
        log::error!("Failed to remove test db: {e}");
    } else {
        log::debug!("Successfully removed test db");
    }
}
