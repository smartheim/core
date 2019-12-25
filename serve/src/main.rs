#![feature(associated_type_defaults)]

mod http;
mod serve_config;

use env_logger::{Env, TimestampPrecision, DEFAULT_FILTER_ENV};
use structopt::StructOpt;
use log::{info, error};

use libohxcore::{common_config, wait_until_known_time, wait_for_root_directory, shutdown_on_ctrl_c, FileFormat, key_filename, cert_filename};
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

    wait_for_root_directory(&config.common.get_root_directory(), false).await?;
    wait_until_known_time(false).await?;
    shutdown_on_ctrl_c(shutdown_tx.clone());

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
        http_shutdown.shutdown().await;
    });

    if let Err(e) = http_service.run().await {
        error!("{}", e);
    }

    let _ = shutdown.await;

    Ok(())
}
