use anyhow::{Result, anyhow};
use clap::{Arg, Command};
use dball_client::{api, daemon::DaemonService, db};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("dball-daemon")
        .version("0.1.0")
        .about("DBall Daemon")
        .arg(
            Arg::new("daemon")
                .long("daemon")
                .short('d')
                .action(clap::ArgAction::SetTrue)
                .help("Run as a daemon process"),
        )
        .arg(
            Arg::new("config-check")
                .long("config-check")
                .action(clap::ArgAction::SetTrue)
                .help("Check configuration and exit"),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .action(clap::ArgAction::Count)
                .help("Set verbose output level"),
        )
        .get_matches();

    let log_level = match matches.get_count("verbose") {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    dball_client::setup(Some(log_level));

    // check configuration if requested
    if matches.get_flag("config-check") {
        return config_check().await;
    }

    run_daemon().await
}

async fn config_check() -> Result<()> {
    log::info!("Checking configuration...");

    // check database connection
    match db::establish_db_connection() {
        Ok(_) => log::debug!("Database connection: OK"),
        Err(e) => {
            log::error!("Database connection failed: {e}");
            return Err(anyhow!("Database connection failed: {e}"));
        }
    }

    // check API configuration
    let api_config = api::ApiConfig::new("api.toml", "api");
    match api_config {
        Ok(_config) => {
            log::info!("API configurations loaded successfully");
            log::debug!("Configuration check: OK");
        }
        Err(e) => {
            log::warn!("Failed to load API configurations: {e}");
        }
    }

    // check socket directory permissions
    let socket_path = std::path::Path::new("/tmp");
    if !socket_path.exists() {
        log::error!("/tmp directory does not exist");
        return Err(anyhow!("/tmp directory does not exist"));
    }

    // try to create a temporary file to check write permission
    let test_file = "/tmp/dball-daemon-test";
    match std::fs::File::create(test_file) {
        Ok(_) => {
            log::debug!("Socket directory permissions: OK");
            std::fs::remove_file(test_file)
                .map_err(|e| {
                    log::warn!("Failed to remove test file: {e}");
                })
                .ok();
        }
        Err(e) => {
            return Err(anyhow!("Socket directory write permission denied: {e}"));
        }
    }

    log::info!("Configuration check completed successfully");
    Ok(())
}

/// run daemon process
async fn run_daemon() -> Result<()> {
    log::info!("Starting DBall daemon...");

    // create daemon service
    let mut daemon_service = DaemonService::new().await?;

    // start service
    daemon_service.start().await?;

    // run main loop
    daemon_service.run().await?;

    // graceful shutdown
    daemon_service.shutdown().await?;

    log::info!("DBall daemon stopped");
    Ok(())
}
