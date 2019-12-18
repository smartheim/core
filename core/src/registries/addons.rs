use semver::Version;
use tokio::io::AsyncBufRead;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_core::Stream;
use futures_core::task::{Context, Poll};
use std::pin::Pin;

#[derive(Clone)]
pub struct AddonInstanceReference<'a> {
    addon_id: &'a str,
    instance_id: &'a str,
    version: Version,
}

//TODO from cloud lib
pub struct AddonAbout {}

pub struct AddonEntry {
    about: AddonAbout,
    instance_id: Option<AddonInstanceWithStatus>,
}

pub enum AddonStatus {
    /// Not running
    Stopped,
    /// In the process of shutting down
    Stopping,
    /// The Addon process itself has been started, but the process has not yet registered with core.
    /// This must happen within 5 seconds after start otherwise core will forcefully stop the process.
    NotYetRegistered,
    /// Starting up. In this status the Addon is allocating resources, loading files etc.
    Starting,
    /// The periodic health check failed last time. Depending on the user configuration this might
    /// cause core to restart the Addon or wait for a few more attempts.
    NonHealthy,
    /// The Addon is up and running.
    Healthy,
}

pub struct AddonManagementOptions {
    /// Restart the addon on failure
    restart_on_failure: bool,
    /// On overall low system memory OHX starts to shutdown Addons. An Addon with a lower priority
    /// is likely to get stopped first.
    low_mem_priority: i32,
    /// Only after this many failed health checks the Addon will be restarted.
    /// A periodic health check is performed every 2 minutes by default.
    /// Additionally a failed Addon communication counts as failed health check.
    failed_health_checks_before_restart: u32,
}

impl Default for AddonManagementOptions {
    fn default() -> Self {
        Self {
            restart_on_failure: true,
            low_mem_priority: 0,
            failed_health_checks_before_restart: 3,
        }
    }
}

//TODO
pub struct AddonInstanceWithStatus {
    instance_id: String,
    status: AddonStatus,
    last_good_health_check: chrono::DateTime<chrono::Utc>,
    failed_health_checks: u32,
    details_i18n: String,
}

//TODO
pub struct StatusEmitter {
    inner: tokio::sync::watch::Receiver<()>,
    abort: Option<std::sync::mpsc::Sender<()>>,
}

impl Drop for StatusEmitter {
    fn drop(&mut self) {
        if let Some(sender) = self.abort.take() {
            let _ = sender.send(());
        }
    }
}

// * List IoServiceInstances of Addons
// * List Things of Addons
// * Execute command on Addon
// * Register to property changes