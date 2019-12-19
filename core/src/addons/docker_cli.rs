use semver::Version;
use tokio::io::AsyncBufRead;

use crate::addons::{AddonManagementOptions, AddonEntry, AddonInstanceReference};
use crate::notifications::StatusEmitter;

struct DockerCLI {}

use async_trait::async_trait;
use crate::addons::management_backend::Backend;

#[async_trait]
impl Backend for DockerCLI {
    async fn get_addon(&self, addon_id: &str, version: Version) -> Option<AddonEntry> {
        unimplemented!()
    }

    async fn get_addon_list(&self) -> Vec<AddonEntry> {
        unimplemented!()
    }

    fn get_log<T>(&self, instance: AddonInstanceReference) -> Option<tokio::io::Lines<T>> where
        T: AsyncBufRead + Unpin {
        unimplemented!()
    }

    fn subscribe_log(&self, instance: AddonInstanceReference) -> StatusEmitter {
        unimplemented!()
    }

    fn stop(&self, instance: AddonInstanceReference) -> StatusEmitter {
        unimplemented!()
    }

    fn start(&self, addon_id: &str, version: Version, options: AddonManagementOptions) -> StatusEmitter {
        unimplemented!()
    }

    fn restart(&self, instance: AddonInstanceReference, options: AddonManagementOptions) -> StatusEmitter {
        unimplemented!()
    }

    fn uninstall(&self, instance: AddonInstanceReference) -> StatusEmitter {
        unimplemented!()
    }

    fn install(&self, addon_id: &str, version: Version) -> StatusEmitter {
        unimplemented!()
    }

    async fn login(&self, username: &str, passphrase: &str, source_id: Option<&str>) -> bool {
        unimplemented!()
    }

    async fn add_source(&self, source: &str) -> bool {
        unimplemented!()
    }
}