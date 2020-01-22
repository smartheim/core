use std::collections::{BTreeMap, HashSet};
use serde::{Serialize,Deserialize};

pub type TokenID = String;

#[derive(Serialize, Deserialize, Debug)]
struct Tokens {
    entries: HashSet<TokenID>
}