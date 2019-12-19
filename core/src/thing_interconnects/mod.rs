//! # The heart of OHX: The interconnections between Things and Things and IOServices.

pub mod interconnects_store;
pub mod thing_templates_registry;
pub mod thing_registry;
pub mod service;

use snafu::{ResultExt, Snafu};
use std::collections::BTreeMap;
use std::slice::Iter;

use libohxcore::acl::Access;
