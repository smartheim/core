use serde::{Serialize, Deserialize};
use schemars::{schema_for, JsonSchema};
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::Arc;

#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
enum RuleMode {
    /// # New Instance
    /// Create a new instance per run
    NewInstancePerRun,
    /// # Singleton
    /// Only one instance of this rule can run at any time.
    /// A start() does nothing if the rule is already running.
    Singleton,
    /// # Singleton, Start New
    /// Only one instance of this rule can run at any time.
    /// A start() will abort an already running rule if any.
    SingletonAbortLast,
}

#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
struct RuleConfig {
    mode: RuleMode
}

#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
struct RuleModuleReference {
    /// # Module ID
    /// The module ID of the target rule module. For example "schedule" for the build-in scheduler.
    module_id: String,
    /// # Addon ID
    /// The addon id of the target rule module. Can be "core" if the rule module is a build-in one.
    addon_id: String,
    /// # Input name mappings
    /// Mappings from an arbitrary custom external input name to the rule module internal input name.
    mapped_required_inputs: BTreeMap<String,String>,
    /// # Output name mappings
    /// Mappings from the rule module internal output name to a custom one
    /// For example for the "schedule" rule module, the "date" output can be renamed to "date.now".
    mapped_provided_outputs: BTreeMap<String,String>,
    /// # Rule module configuration
    /// Some rule modules require configuration.
    /// The scheduler module for example needs to know when to trigger.
    config: BTreeMap<String, serde_json::Value>,
}


struct ModuleInstance {
    // TODO addon connection gRPC / tonic
    connection: String,

}

#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
struct RuleActionWithConditionAndChildren {
    children: Vec<Box<RuleActionWithConditionAndChildren>>,

    conditions: Vec<RuleModuleReference>,
    action: RuleModuleReference,
}

#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
struct Rule {
    /// # Unique ID
    /// The rule ID must be unique amongst all rules.
    /// The ID is used as filename as well.
    id: String,
    /// # Configuration
    /// The rule configuration
    config: RuleConfig,
    /// # Rule Meta Data
    about: libohx::meta::MetaInformation,
    /// # Actions
    actions: Vec<RuleActionWithConditionAndChildren>,
    /// # Triggers
    triggers: Vec<RuleModuleReference>,
}

/// If dropped will unregister the trigger.
struct RuleTriggerController {}

struct RuleInstance {
    rule: Arc<Rule>,
    module_instances: HashMap<String,ModuleInstance>,
    triggers: Vec<RuleTriggerController>
}