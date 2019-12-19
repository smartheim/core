#![feature(associated_type_defaults)]

mod errors;
mod addons;
mod ioservices;
mod ipc;
mod thing_interconnects;
mod core_config;
mod notifications;

use env_logger::{Env, TimestampPrecision, DEFAULT_FILTER_ENV};
use std::path::Path;
use structopt::StructOpt;
use log::{info, error};
use snafu::Error;
use futures_util::future::select;

use libohxcore::{common_config, wait_until_known_time};
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
    let config: core_config::Config = core_config::Config::from_args();
    let common_config: common_config::Config = common_config::Config::from_args();

    create_root_directory(&common_config, &config)?;
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
    shutdown_tx.send(()).await.unwrap();
    Ok(())
}

fn create_root_directory(common_config: &common_config::Config, config: &core_config::Config) -> Result<(), std::io::Error> {
    let path = common_config.get_root_directory();
    if !config.create_root && !path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "OHX Root directory does not exist. Consider using --create-root").into());
    }

    std::fs::create_dir_all(path.join("addons_http"))?;
    std::fs::create_dir_all(path.join("backups"))?;
    std::fs::create_dir_all(path.join("certs"))?;
    std::fs::create_dir_all(path.join("config"))?;
    std::fs::create_dir_all(path.join("interconnects"))?;
    std::fs::create_dir_all(path.join("rules"))?;
    std::fs::create_dir_all(path.join("scripts"))?;
    std::fs::create_dir_all(path.join("webui"))?;
    Ok(())
}
