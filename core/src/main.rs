#![feature(associated_type_defaults)]

mod errors;
mod addons;
mod ioservices;
mod ipc;
mod thing_interconnects;
mod core_config;
mod notifications;
mod create_http_certificate;
mod create_system_user;

use env_logger::{Env, TimestampPrecision, DEFAULT_FILTER_ENV};
use std::path::Path;
use structopt::StructOpt;
use log::{info, error};
use snafu::Error;

use libohxcore::{common_config, wait_until_known_time, shutdown_on_ctrl_c};
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
    let (mut shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(1);

    shutdown_on_ctrl_c(shutdown_tx.clone());
    create_root_directory(&config.common, &config)?;
    create_system_user::create_check_system_user(&config.common.get_certs_directory())?;
    wait_until_known_time(false).await?;

    // Check and create self signed cert. Start certificate refresh task with graceful shutdown warp channel
    create_http_certificate::check_gen_certificates(&config.common.get_certs_directory())?;
    let (certificate_refresher, mut cert_watch_shutdown_tx) = create_http_certificate::RefreshSelfSigned::new(config.common.get_certs_directory());
    tokio::spawn(async move { certificate_refresher.run().await; });

    // Shutdown task
    let mut shutdown = tokio::spawn(async move {
        let _ = shutdown_rx.recv().await;
        let _ = cert_watch_shutdown_tx.send(());
    });

    let _ = tokio::time::delay_for(Duration::from_secs(3)).await;
    shutdown_tx.send(()).await.unwrap();

    let _ = shutdown.await;
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
