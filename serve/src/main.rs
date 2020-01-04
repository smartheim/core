#![feature(associated_type_defaults)]

mod http;
mod serve_config;
mod create_http_certificate;

use env_logger::{Env, TimestampPrecision, DEFAULT_FILTER_ENV};
use structopt::StructOpt;
use log::{info, error};

use libohxcore::{common_config, wait_until_known_time, shutdown_on_ctrl_c, FileFormat, key_filename, cert_filename};
use http::service::HttpService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logging
    let mut builder = env_logger::Builder::from_env(Env::new().filter_or(DEFAULT_FILTER_ENV, "info"));
    builder
        .format_timestamp(Some(TimestampPrecision::Seconds))
        .format_module_path(false)
        .init();

    // Command line / environment / file configuration
    let config: serve_config::Config = serve_config::Config::from_args();
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(1);

    create_root_directory(&config.common)?;
    wait_until_known_time(false).await?;
    shutdown_on_ctrl_c(shutdown_tx.clone());

    let config_dir = config.common.get_service_config_directory("ohx-serve")?;

    // Check and create self signed cert. Start certificate refresh task with graceful shutdown warp channel
    create_http_certificate::check_generate(&config.common.get_certs_directory(), &config_dir)?;
    let (certificate_refresher, mut cert_watch_shutdown_tx) = create_http_certificate::RefreshSelfSigned::new(config.common.get_certs_directory(),config_dir.to_path_buf());
    tokio::spawn(async move { certificate_refresher.run().await; });

    let mut http_service = HttpService::new(config.common.get_root_directory(),
                                            key_filename(&config.common.get_certs_directory(), FileFormat::PEM),
                                            cert_filename(&config.common.get_certs_directory(), FileFormat::PEM));

    let entries = http_service.redirect_entries();
    entries.add("core".to_owned(), "192.168.1.3".to_owned(), "common".to_owned());
    let entries = http_service.redirect_entries();
    entries.add("core".to_owned(), "192.168.1.3".to_owned(), "general".to_owned());

    // Shutdown task
    let http_shutdown = http_service.control();
    let mut shutdown = tokio::spawn(async move {
        let _ = shutdown_rx.recv().await;
        let _ = cert_watch_shutdown_tx.send(());
        http_shutdown.shutdown().await;
    });

    if let Err(e) = http_service.run().await {
        error!("{}", e);
    }

    let _ = shutdown.await;

    Ok(())
}

/// Creates all OHX root directory subdirectories required to run the OHX serve service
fn create_root_directory(common_config: &common_config::Config) -> Result<(), std::io::Error> {
    let path = common_config.get_root_directory();
    if !common_config.create_root && !path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "OHX Root directory does not exist. Consider using --create-root").into());
    }

    // The generic config directory
    std::fs::create_dir_all(path.join("config"))?;

    // Directories used by this service
    std::fs::create_dir_all(path.join("addons_http"))?;
    std::fs::create_dir_all(path.join("certs"))?;
    std::fs::create_dir_all(path.join("webui"))?;

    // Served directories
    std::fs::create_dir_all(path.join("backups"))?;
    std::fs::create_dir_all(path.join("interconnects"))?;
    std::fs::create_dir_all(path.join("rules"))?;
    std::fs::create_dir_all(path.join("scripts"))?;

    Ok(())
}
