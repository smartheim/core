//! # User provided meta information for Rules, Things, Interconnections.

use serde::{Serialize, Deserialize};
use schemars::{JsonSchema};

/// User provided meta information for something.
/// Something can be a Thing, an Interconnection, a Rule.
///
/// Meta information is non-translated data like a title, a description, tags
/// and is used for presentation and searching.
#[derive(JsonSchema)]
#[derive(Serialize, Deserialize)]
pub struct MetaInformation {
    /// # Title
    /// A title for this object
    title: String,
    /// # Description
    /// An optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// # Tags
    /// Tag this object with keywords. For example "Summer Only" or "Moody".
    /// Tags are case sensitive and may contain one or more words.
    ///
    /// Tags are used for searching, but can also be used to group objects in rules,
    /// eg to switch on all lights with the tag "moody".
    tags: Vec<String>
}