//! # Multiple core registries
//!
//! A registry usually has a way to serialize itself periodically or on specific events
//! and deserialize on startup. A registry is used as cache to respond to list-all queries,
//! and also serves as a bridge to API that calls into Addons or is populated by Addons in the first place.
pub mod addons;
pub mod ioservice_template;
pub mod schemas;
pub mod thing_templates;
pub mod things;
pub mod webuis;