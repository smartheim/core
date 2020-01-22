//! # The configurable trait
//! A configurable type

use schemars::schema::Schema;
pub use schemars::JsonSchema;
pub use schemars;

use serde::{Serialize, Deserialize};
use tokio::sync::mpsc::{Sender, Receiver};

use crate::schema_registry_trait::{SchemaRegistryTrait, SchemaRegistryCommand};
use super::watcher::ConfigWatcherCommand;
use std::sync::Arc;

pub trait Configurable: JsonSchema + Serialize + Default {
    /// Default implementation. Returns the auto-generated schema.
    /// Can be overridden with a custom schema string.
    fn schema(&self) -> String {
        let schema = schemars::gen::SchemaGenerator::default().into_root_schema_for::<Self>();
        serde_json::to_string(&schema).expect("Valid jsonSchema annotations")
    }
    fn serialized(&self) -> String {
        serde_json::to_string(&self).expect("Valid jsonSchema annotations")
    }
}

pub struct ConfigWithSchemaManager {
    /// The communication channel to notify about schema related events
    schema_changed: Sender<SchemaRegistryCommand>,
    /// The communication channel to notify about configuration related events
    config_changed: Sender<ConfigWatcherCommand>,
    schema_name: String,
}

impl ConfigWithSchemaManager {
    pub fn new(schema_registry: &impl SchemaRegistryTrait, config_changed: Sender<ConfigWatcherCommand>, schema_name: String) -> Self {
        Self {
            schema_changed: schema_registry.get_changed_channel(),
            config_changed,
            schema_name: schema_name,
        }
    }
    pub async fn schema_changed(&self, configurable: impl Configurable) {
        self.schema_changed.clone().send(SchemaRegistryCommand::Register {
            schema_name: self.schema_name.clone(),
            schema_serialized: configurable.schema(),
        }).await;
    }
    pub async fn config_changed(&self, configurable: impl Configurable) {
        self.config_changed.clone().send(ConfigWatcherCommand::WriteChange {
            schema_name: self.schema_name.clone(),
            config_serialized: configurable.serialized(),
        }).await;
    }
}

impl Drop for ConfigWithSchemaManager {
    fn drop(&mut self) {
        let schema_name = self.schema_name.clone();
        let mut channel = self.schema_changed.clone();
        tokio::spawn(async move {
            let _ = channel.send(SchemaRegistryCommand::Unregister(schema_name)).await;
        });
    }
}
