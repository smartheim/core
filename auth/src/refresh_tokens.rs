use std::collections::{BTreeMap, HashSet};

pub type TokenID = String;

#[derive(Serialize, Deserialize, Debug)]
struct Tokens {
    entries: HashSet<TokenID>
}