use std::collections::{HashMap, BTreeMap};
use generational_arena::Arena;
use arc_swap::ArcSwap;
use libohxcore::command::Command;
use libohxcore::property::{Property, PropertyReference, ThingReference};
use std::sync::RwLock;
use libohxcore::addon::connection::AddonConnection;
use libohxcore::meta::MetaInformation;

/// An addon registers available, auto-discovered Things as well as manually configured Things on each start.
/// Core does not store those anywhere across reboots.
/// Core does store a PropertyReference (containing a ThingUID) in the Interconnect Store though.
struct ThingRegistry {
    /// Things are stored in a generational Arena.
    /// They are not accessed via a pointer/reference but via arena coordinates
    store: Arena<Thing>,
    mapping_things: HashMap<ThingUID, ThingReference>,
}

impl ThingRegistry {
    /// A thing is stored in the ThingRegistry with a certain runtime index.
    pub fn add_thing(&self) {
        let arena = self.things.write().unwrap();
        arena.insert()
    }
}

/// A unique identifier of a Thing across all addons.
/// Archived by concatenating addon_id and thing_id: addon_id/things/thing_id.
/// This is to be in line with https://iot.mozilla.org/wot/ and their Thing URIs: https://someurl.com/things/switch
///
/// The actual length of the thing UID is not that important.
/// It is exposed for IO Addons that require a unique identifier like the Web of Things API IO Addon.
/// Internally in OHX Core an integer based reference is used to reference a Thing instead of the Thing UID string.
#[derive(Serialize, Deserialize)]
struct ThingUID(String);

impl ThingUID {
    pub fn new(addon_id: &str, thing_id: &str) -> Self {
        const THINGS_STR: &'static str = "/things/";
        let mut id = String::with_capacity(addon_id.len() + thing_id.len() + THINGS_STR.len());
        id += addon_id;
        id += THINGS_STR;
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

/// The Thing data transfer object (DTO) that is send over the wire during Addon registration
/// when Things are shared with OHX Core.
#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
struct ThingDTO {
    id: String,
    #[serde(flatten)]
    meta: MetaInformation,
    properties: BTreeMap<String, Property>,
}

/// A Thing.
struct Thing {
    uid: ThingUID,
    #[serde(flatten)]
    meta: MetaInformation,
    properties: Vec<Property>,
    #[serde(default)]
    #[schemars(skip)]
    index_for_property_id: Vec<String>,
    addon: AddonConnection,
}

impl Thing {
    pub fn new(addon: AddonConnection) -> Self {
        Self {
            properties: vec![],
            index_for_property_id: vec![],
            uid: ThingUID(),
            addon: addon,
        }
    }
    /// Called when the addon (re-)registers to the AddonRegistry.
    pub fn addon_connection_changed(&mut self, addon: AddonConnection) {
        self.addon = addon;
    }
    /// RPC Call to the Addon to transmit the new value. The returned value will be stored in the Thing again.
    /// This method is faster if the
    pub fn update_property(&mut self, index: PropertyReference, value: Property) {
        if let Some(prop) = self.properties.get_mut(index.property_instance_id) {
            prop.replace(value);
        }
    }
}

/// Return a [`PropertyReference`] for a given property name or None if no such property exists.
pub fn get_prop_reference(thing: &Thing, reference: ThingReference, name: &str) -> Option<PropertyReference> {
    if let Ok(property_instance_id) = thing.index_for_property_id.binary_search_by(|v| v.as_str().cmp(name)) {
        return Some(PropertyReference { thing_instance_id: reference.clone(), property_instance_id: property_instance_id as u64 });
    }
    None
}