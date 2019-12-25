use std::collections::HashMap;
use generational_arena::Arena;
use arc_swap::ArcSwap;
use libohxcore::command::Command;
use libohxcore::property::Property;

/// Addons register available, auto-discovered Things as well as manually configured Things on each start.
/// Core does not store those anywhere across reboots or Addon disconnect/connect cycles.
/// Core does store a PropertyReference (containing a ThingUID) in the Interconnect Store though.
struct ThingRegistry {
    /// Things are stored in a generational Arena.
    /// They are not accessed via
    things: ArcSwap<Arena<Thing>>,

    config: Config,
}

/// A unique identifier of a Thing across all addons.
/// Archived by concatenating addon_id and thing_id: addon_id#thing_id
struct ThingUID(String);

impl ThingUID {
    pub fn new(addon_id: &str, thing_id: &str) -> Self {
        let mut id = String::with_capacity(addon_id.len() + thing_id.len() + 1);
        id += addon_id;
        id += "#";
        id += thing_id;
        ThingUID(id)
    }
}

struct Config {
    hidden_things: ArcSwap<Vec<ThingUID>>
}

struct ThingRegistryProcessor {}

impl ThingRegistryProcessor {
    pub async fn run() {}
}

/// A Thing. TODO json schema
struct Thing {
    property_indices: Vec<Property>,
    index_for_property_id: Vec<String>,
    thing_uid:String
}

impl Thing {
    pub fn new() {

    }
}