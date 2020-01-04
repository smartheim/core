pub mod common_config;
pub mod command;
pub mod property;
pub mod meta;
pub mod addon;
pub mod acl;
pub mod configurable;

use log::{info, warn};

use chrono::Datelike;
use std::path::{Path, PathBuf};

pub use biscuit;

/// Wait until time is known.
/// Systems without a buffered clock will start with unix timestamp 0 (1970/1/1) and that will break
/// certificate validation and signing.
pub async fn wait_until_known_time(no_wait: bool) -> Result<(), std::io::Error> {
    let mut now = chrono::Utc::now();
    if now.year() == 1970 { warn!("Waiting for correct date/time..."); }
    while now.year() == 1970 {
        if no_wait { return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Time unknown")); }
        tokio::time::delay_for(std::time::Duration::from_secs(2)).await;
        now = chrono::Utc::now();
    }
    Ok(())
}

pub async fn wait_for_root_directory(root_dir:&Path, no_wait: bool)-> Result<(), std::io::Error> {
    let sys_user_pub = root_dir.join("certs/ohx_pub_sys.pem");
    let sys_user_priv = root_dir.join("certs/ohx_priv_sys.pem");
    let cert_pub = root_dir.join("certs/https_key.pem");
    let cert_priv = root_dir.join("certs/https_cert.pem");

    if !root_dir.exists() { warn!("Waiting for root directory..."); }
    if !sys_user_pub.exists() { warn!("certs/ohx_pub_sys.pem does not exist..."); }
    if !sys_user_priv.exists() { warn!("certs/ohx_priv_sys.pem does not exist..."); }
    if !cert_pub.exists() { warn!("certs/https_key.pem does not exist..."); }
    if !cert_priv.exists() { warn!("certs/https_cert.pem does not exist..."); }

    while !root_dir.exists() || !cert_pub.exists() || !cert_priv.exists() || !sys_user_pub.exists() || !sys_user_priv.exists() {
        if no_wait { return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Root directory not found")); }
        tokio::time::delay_for(std::time::Duration::from_secs(2)).await;
    }
    Ok(())
}

/// Spawns a task onto the current tokio executor that notifies the given shutdown channel whenever
/// ctrl+c has been issued to the process.
pub fn shutdown_on_ctrl_c(mut shutdown_tx: tokio::sync::mpsc::Sender<()>) {
    tokio::spawn(async move {
        loop {
            let _ = tokio::signal::ctrl_c().await;
            info!("Ctrl+C: Shutting down");
            shutdown_tx.send(()).await.unwrap();
        }
    });
}

const KEY_FILENAME: &'static str = "https_key.pem";
const KEY_FILENAME_DER: &'static str = "https_key.der";
const PUBLIC_FILENAME: &'static str = "https_cert.pem";
const PUBLIC_FILENAME_DER: &'static str = "https_cert.der";

const SYSTEM_AUTH_JKWS: &'static str = "ohx_system.jwks";

pub enum FileFormat {
    DER,
    PEM,
}

pub fn key_filename(cert_dir: &Path, format: FileFormat) -> PathBuf {
    match format {
        FileFormat::DER => cert_dir.join(KEY_FILENAME_DER),
        FileFormat::PEM => cert_dir.join(KEY_FILENAME),
    }
}

pub fn cert_filename(cert_dir: &Path, format: FileFormat) -> PathBuf {
    match format {
        FileFormat::DER => cert_dir.join(PUBLIC_FILENAME_DER),
        FileFormat::PEM => cert_dir.join(PUBLIC_FILENAME),
    }
}

pub fn system_auth_jwks(cert_dir: &Path) -> PathBuf {
    cert_dir.join(SYSTEM_AUTH_JKWS)
}
