use valico::json_schema::Schema;
use std::collections::{BTreeMap};

struct SchemaRegistry {
    schemas: BTreeMap<String, Schema>
}