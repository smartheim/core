pub mod common_config;
pub mod command;
pub mod discovery;
pub mod meta;
pub mod addon;
pub mod acl;

use log::warn;

use chrono::Datelike;
use std::thread::sleep;
use std::path::Path;

/// Wait until time is known.
/// Systems without a buffered clock will start with unix timestamp 0 (1970/1/1) and that will break
/// certificate validation and signing.
pub fn wait_until_known_time(no_wait: bool) -> Result<(), std::io::Error> {
    let mut now = chrono::Utc::now();
    if now.year() == 1970 { warn!("Waiting for correct date/time..."); }
    while now.year() == 1970 {
        if no_wait { return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Time unknown")); }
        sleep(std::time::Duration::from_secs(2));
        now = chrono::Utc::now();
    }
    Ok(())
}

pub fn wait_for_root_directory(root_dir:&Path, no_wait: bool)-> Result<(), std::io::Error> {
    if !root_dir.exists() { warn!("Waiting for root directory..."); }
    let cert_pub = root_dir.join("certs/ohx_pub_sys.der");
    let cert_priv = root_dir.join("certs/ohx_priv_sys.der");
    while !root_dir.exists() || !cert_pub.exists() || !cert_priv.exists() {
        if no_wait { return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Root directory not found")); }
        sleep(std::time::Duration::from_secs(2));
    }
    Ok(())
}