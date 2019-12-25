use crate::acl::UserID;

pub struct ThingInstanceUid(usize, u64);

pub struct Property {
    value: serde_json::Value
}

pub struct PropertyReference {
    thing_instance_id: ThingInstanceUid,
    property_instance_id: u64,
    thing_uid: String,
    property_id: String
}