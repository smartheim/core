pub mod common_config;
pub mod command;
pub mod property;
pub mod meta;
pub mod addon;
pub mod acl;

use log::{info, warn};

use chrono::Datelike;
use std::path::{Path, PathBuf};

use std::fs::File;

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


// Check if given directory is writable
pub fn check_dir_access(path: &Path) -> Result<(), std::io::Error> {
    let dummy_file = path.join("_non_");
    let _ = File::create(&dummy_file)?;
    std::fs::remove_file(&dummy_file)?;
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
