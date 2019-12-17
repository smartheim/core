#![feature(associated_type_defaults)]

mod certificates;
mod http_server;
mod errors;
mod addon_manage_backends;
mod registries;
mod ipc;
mod interconnects;

use env_logger::{Env, TimestampPrecision, DEFAULT_FILTER_ENV};
use std::path::{Path};
use structopt::StructOpt;
use log::{info,error};

use libohx::{core_config, wait_until_known_time};


fn create_root_directory(dir: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dir.join("addons_http"))?;
    std::fs::create_dir_all(dir.join("backups"))?;
    std::fs::create_dir_all(dir.join("certs"))?;
    std::fs::create_dir_all(dir.join("config"))?;
    std::fs::create_dir_all(dir.join("interconnects"))?;
    std::fs::create_dir_all(dir.join("rules"))?;
    std::fs::create_dir_all(dir.join("scripts"))?;
    std::fs::create_dir_all(dir.join("webui"))?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = env_logger::Builder::from_env(Env::new().filter_or(DEFAULT_FILTER_ENV, "info"));
    builder
        .format_timestamp(Some(TimestampPrecision::Seconds))
        .format_module_path(false)
        .init();

    let config: core_config::Config = core_config::Config::from_args();

    let path = config.get_root_directory();
    if !config.create_root && !path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "OHX Root directory does not exist. Consider using --create-root").into());
    }
    create_root_directory(&path)?;

    wait_until_known_time(false)?;

    // Create certificates
    let cert_dir = path.join("certs");
    certificates::check_gen_certificates(&cert_dir)?;

    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(1);

    use http_server::HttpService;
    let mut http_service = HttpService::new(config.get_root_directory());

    let entries = http_service.redirect_entries();
    entries.add("core".to_owned(),"192.168.1.3".to_owned(),"common".to_owned());
    let entries = http_service.redirect_entries();
    entries.add("core".to_owned(),"192.168.1.3".to_owned(),"general".to_owned());

    // Start certificate refresh task with graceful shutdown warp channel
    tokio::spawn(async {});

    let http_shutdown = http_service.control();
    tokio::spawn(async move {
        shutdown_rx.recv().await;
        http_shutdown.shutdown().await;
    });

    // Ctrl+C task
    let mut shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        loop {
            let _ = tokio::signal::ctrl_c().await;
            info!("Ctrl+C: Shutting down");
            shutdown_tx_clone.send(()).await.unwrap();
        }
    });

    if let Err(e) = http_service.run().await {
        error!("{}",e);
    }

//    let mut shutdown_tx_clone = shutdown_tx.clone();
//    tokio::spawn(async move {
//        loop {
//            let _ = tokio::time::delay_for(Duration::from_secs(3)).await;
//            info!("Timeout: Shutting down");
//            shutdown_tx_clone.send(()).await.unwrap();
//        }
//    });

    Ok(())
}
