use tokio::sync::mpsc::{Sender, Receiver};

pub enum SchemaRegistryCommand {
    /// Registers a schema with the given name to the schema registry
    Register { schema_name: String, schema_serialized: String },
    Unregister(String),
}

/// A schema registry is either the real SchemaRegistry if within ohx-core,
/// or an RPC proxy.
pub trait SchemaRegistryTrait {
    fn get_changed_channel(&self) -> Sender<SchemaRegistryCommand>;
}