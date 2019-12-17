mod docker_cli;

use async_trait::async_trait;
use semver::Version;
use tokio::io::AsyncBufRead;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use crate::registries::addons::{AddonEntry, StatusEmitter, AddonManagementOptions, AddonInstanceReference};

#[async_trait]
trait Backend {
    async fn get_addon(&self, addon_id: &str, version: Version) -> Option<AddonEntry>;
    async fn get_addon_list(&self) -> Vec<AddonEntry>;

    fn get_log<T>(&self, instance: AddonInstanceReference) -> Option<tokio::io::Lines<T>> where
        T: AsyncBufRead + Unpin;
    /// Subscribe to the log. Unsubscribe by canceling or dropping the status emitter.
    fn subscribe_log(&self, instance: AddonInstanceReference) -> StatusEmitter;

    /// Stops the Addon referenced by the instance ID
    fn stop(&self, instance: AddonInstanceReference) -> StatusEmitter;
    /// Starts the Addon referenced by the ID and version with the given options.
    fn start(&self, addon_id: &str, version: Version, options: AddonManagementOptions) -> StatusEmitter;
    /// Restarts the Addon referenced by the instance ID
    fn restart(&self, instance: AddonInstanceReference, options: AddonManagementOptions) -> StatusEmitter;

    /// Uninstalls the Addon referenced by the instance ID
    fn uninstall(&self, instance: AddonInstanceReference) -> StatusEmitter;

    /// Installs the Addon referenced by the ID and version.
    fn install(&self, addon_id: &str, version: Version) -> StatusEmitter;

    /// Logs in to the backend to install/uninstall Addons.
    /// Not all backends require this.
    async fn login(&self, username: &str, passphrase: &str, source_id: Option<&str>) -> bool;
    /// Adds a source for getting Addons to this backend. For docker this would be a docker compatible Registry.
    async fn add_source(&self, source: &str) -> bool;
}