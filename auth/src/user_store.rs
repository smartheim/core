use serde::{Serialize, Deserialize};
use libohxcore::acl::{AccessScopes, UserID};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: UserID,
    name: Option<String>,
    scopes: Vec<AccessScopes>,
}

struct UserStore {
    entries: BTreeMap<UserID, User>
}

// impl StoreInSeparateFiles<T>