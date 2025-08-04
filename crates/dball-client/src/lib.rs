use std::{path::PathBuf, sync::LazyLock};

pub(crate) static ENV_GUARD: LazyLock<Result<PathBuf, anyhow::Error>> = LazyLock::new(|| {
    dotenvy::dotenv().map_err(|e| anyhow::anyhow!("Failed to load .env file: {e}"))
});

pub mod api;
pub mod daemon;
pub mod db;
pub mod ipc;
pub mod models;
pub mod service;

const NEVER_NONE_BY_DATABASE: &str = "Should not be None guaranteed by database";

pub fn setup(log_level: Option<log::LevelFilter>) {
    init_env();

    let mut logger = env_logger::Builder::from_default_env();
    if let Some(level) = log_level {
        logger.filter_level(level);
    }

    logger.try_init().expect("Failed to initialize logger");
}

/// load env file, panic if failed
fn init_env() {
    crate::ENV_GUARD
        .as_ref()
        .expect("Failed to load environment variables. Ensure .env file exists and is correctly configured.");
}

pub(crate) fn parse_from_env<T: std::str::FromStr>(key: &str) -> Option<T>
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let Some(value) = std::env::var(key).ok() else {
        log::warn!("Environment variable {key} not set, returning None");
        return None;
    };

    value.parse::<T>().ok().or_else(|| {
        log::error!("Failed to parse {key} from env, returning None");
        None
    })
}

#[ctor::ctor]
#[cfg(test)]
fn init_test_logger() {
    init_env();

    eprintln!("Initializing test logger");
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
    let root_path = crate::ENV_GUARD
        .as_ref()
        .expect("ENV_GUARD not initialized")
        .parent()
        .expect("env parent path not found");
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
        if file.exists() && std::fs::remove_file(file).is_err() {
            log::debug!("Failed to remove old test db: {}", file.display());
        }
    }

    // copy main database as test database
    if std::fs::copy(main_db_path, &test_db_path).is_err() {
        log::error!(
            "Failed to copy main db to test db: {}",
            test_db_path.display()
        );
    }

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
