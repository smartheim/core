//! # Build-in modules: Rule engine modules are referenced by rules and are the executing building blocks of the engine.
//! A scheduler, property changes, rule changes, addon changes are included and can be used as triggers.
//! Properties can be checked against for conditions. Commands can be issued as actions.
//! The rule engine can also be controlled via build-in modules.
//!
//! A bunch of transformations allow to extract values out of structured formats like json,
//! convert between strings and numbers,
//! perform mathematical operations (add, subtract, multiply, divide, modulo, offset) on numbers and
//! operations on strings.
pub mod schedule;