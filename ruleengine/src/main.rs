mod rule_config;
mod rule;
mod rules_registry;
mod modules_registry;
mod buildin_modules;
mod engine;

use env_logger::{Env, TimestampPrecision, DEFAULT_FILTER_ENV};
use std::path::Path;
use structopt::StructOpt;
use log::{info, error};

use libohx::{wait_until_known_time, wait_for_root_directory, common_config};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logging
    let mut builder = env_logger::Builder::from_env(Env::new().filter_or(DEFAULT_FILTER_ENV, "info"));
    builder
        .format_timestamp(Some(TimestampPrecision::Seconds))
        .format_module_path(false)
        .init();

    // Command line / environment / file configuration
    let config: rule_config::Config = rule_config::Config::from_args();
    let common_config: common_config::Config = common_config::Config::from_args();

    let path = common_config.get_root_directory();
    wait_for_root_directory(&path, false)?;
    wait_until_known_time(false)?;

    let (mut shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(1);

    // Ctrl+C task
    let mut shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        loop {
            let _ = tokio::signal::ctrl_c().await;
            info!("Ctrl+C: Shutting down");
            shutdown_tx_clone.send(()).await.unwrap();
        }
    });

    let _ = tokio::time::delay_for(Duration::from_secs(3)).await;
    info!("Timeout: Shutting down");
    shutdown_tx.send(()).await.unwrap();

    Ok(())
}
