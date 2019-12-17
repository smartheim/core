pub mod core_config;
pub mod addon_startup_config;
pub mod discovery;

use chrono::Datelike;
use std::thread::sleep;

pub fn wait_until_known_time(no_time_wait: bool) -> Result<(), std::io::Error> {
    // Wait until time is known. Systems without a buffered clock will start with unix timestamp 0 (1970/1/1).
    let mut now = chrono::Utc::now();
    while now.year() == 1970 {
        if no_time_wait { return Err(std::io::Error::new(std::io::ErrorKind::Other, "Time unknown")); }
        sleep(std::time::Duration::from_secs(2));
        now = chrono::Utc::now();
    }
    Ok(())
}
