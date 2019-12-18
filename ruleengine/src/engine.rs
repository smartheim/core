use std::collections::HashMap;
use std::sync::{Mutex, Arc};

/// Keeps track of the run state of rules, propagates states to observers,
/// maintains global variables.
struct RuleEngine {
    global_variables: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}