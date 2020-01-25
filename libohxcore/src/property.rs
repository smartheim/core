#[derive(Clone)]
pub struct ThingReference(usize, u64);

impl ThingReference {
    /// Creates an invalid thing reference
    pub fn invalid() -> Self {
        Self{ 0: 0, 1: 0 }
    }
}

#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
pub struct Property {
    value: serde_json::Value
}

impl Property {
    pub fn replace(&mut self, other: Property) {
        self.value = other.value;
    }
}

/// A property reference allows for a fast access to the referenced Thing and Property,
/// but may not be valid anymore if an Addon has replaced the referenced Thing since.
///
/// All methods that take a [`PropertyReference`] will indicate a non valid reference via
/// an error code. In such a case the [`PropertyCoordinates`] need to be used to get a new
/// valid [`PropertyReference`].
pub struct PropertyReference {
    pub thing_instance_id: ThingReference,
    pub property_instance_id: u64,
}

/// Uniquely identifies a Property of a certain Thing.
/// You usually want to retrieve a [`PropertyReference`] for [`PropertyCoordinates`]
/// instead of using this type directly.
pub struct PropertyCoordinates {
    thing_uid: String,
    property_id: String,
}