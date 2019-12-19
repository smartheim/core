//! # IOService related functionality
//! An IOService provides access for external entities.
//! For example the WebThing-IOService provides the WebThing API for OHX and allows clients to interact
//! with OHX via that API. The cloud-connector IOService mediates between Google Home, Alexa and OHX.

pub mod ioservice_template_registry;
pub mod ioservice_registry;
pub mod ioservice_store;

use libohxcore::acl::{Access, UserID, self};
use std::collections::BTreeMap;


pub type IOServiceInstanceID = String;

pub type FilterPropPipe = Vec<FilterProp>;

pub struct FilterProp {}

pub type FilterCommandPipe = Vec<FilterCommand>;

pub struct FilterCommand {}

pub struct Entry {
    /// Filters for outgoing values
    filter_property_pipe: FilterPropPipe,
    /// Filters for incoming values
    command_filter_pipe: FilterCommandPipe,
    /// Who is able to edit this interconnection entry?
    access: Access,
}

pub struct PropertyValue(serde_json::Value);

pub struct IOServiceInterconnect {
    connections: BTreeMap<IOServiceInstanceID, Entry>,
    command_receiver: tokio::sync::mpsc::Receiver<serde_json::Value>,
    command_sender: tokio::sync::mpsc::Sender<serde_json::Value>,
}

pub struct IOServiceCommandPublisher {
    command_sender: tokio::sync::mpsc::Sender<serde_json::Value>,
}


/// Future
/// * on file change -> reload
/// * command_receive -> AddonRegistry.exec_command
/// *
///
/// * store on update/remove without reload
impl IOServiceInterconnect {
    pub fn new() -> Self {
        let (command_sender, command_receiver) = tokio::sync::mpsc::channel::<serde_json::Value>(1);
        IOServiceInterconnect { connections: Default::default(), command_receiver, command_sender }
    }

    pub fn store(&self) {}

    pub fn load(&mut self) {}

    pub async fn property_changed(&mut self, addon_id: &str, thing_uid: &str, prop_name: &str, context_properties: BTreeMap<String,PropertyValue>) {}

    pub fn command_publisher(&self) -> IOServiceCommandPublisher {
        IOServiceCommandPublisher { command_sender: self.command_sender.clone() }
    }

    pub fn update(&mut self, user: UserID, instance_id: &str, filter_property_pipe: FilterPropPipe, command_filter_pipe: FilterCommandPipe) -> Result<(), acl::Error> {
        Ok(())
    }

    pub fn remove(&mut self, user: UserID, instance_id: &str) -> Result<(), acl::Error> {
        Ok(())
    }
}

pub struct Interconnect {

}