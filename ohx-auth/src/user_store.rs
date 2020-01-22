use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use arc_swap::ArcSwap;
use tokio::sync::mpsc::{Sender, Receiver};

use libohxaddon::users::{UserID, AccessScopes};
use libohxaddon::config::{Configurable, ConfigWithSchemaManager, JsonSchema,schemars};
use libohxaddon::schema_registry_trait::SchemaRegistryTrait;
use libohxaddon::config::watcher::{ConfigurationWatcher, ConfigChangedCommand, ConfigWatcherCommand, self};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: UserID,
    name: Option<String>,
    scopes: Vec<AccessScopes>,
}

#[derive(JsonSchema, Serialize, Deserialize, Default, Debug)]
struct Config {}

impl Configurable for Config {}

struct UserStore {
    entries: Mutex<BTreeMap<UserID, User>>,
    config: ArcSwap<Config>,
    config_manager: ConfigWithSchemaManager,
}

pub struct UserStoreSync(Arc<UserStore>);

/// Start user store
pub fn run(schema_registry: &impl SchemaRegistryTrait, config_watcher: &mut ConfigurationWatcher) -> Result<UserStoreSync, watcher::Error> {
    let (sender_changed, receiver_changed) = tokio::sync::mpsc::channel::<ConfigChangedCommand>(1);
    let (sender_watcher, receiver_watcher) = tokio::sync::mpsc::channel::<ConfigWatcherCommand>(1);


    let config: Config = config_watcher.register(sender_changed)?;

    let subject = UserStoreSync(Arc::new(UserStore {
        entries: Mutex::new(BTreeMap::default()),
        config: ArcSwap::new(Arc::new(config)),
        config_manager: ConfigWithSchemaManager::new(schema_registry, sender_watcher, Config::schema_name()),
    }));

    //subject.0.config_manager.schema_changed(subject.0.config.load()).await;
    //tokio::spawn(async move {});

    Ok(subject)
}
// impl StoreInSeparateFiles<T>