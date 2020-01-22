use strum_macros::IntoStaticStr;
use std::fmt::Debug;
use serde::export::Formatter;
use serde::{Serialize, Deserialize};
use std::cmp::Ordering;

/// A user has an ID that must be a valid filename (passes filename sanitizer).
#[derive(Serialize, Deserialize, Debug)]
pub struct UserID(String);

impl std::cmp::Ord for UserID {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl std::cmp::PartialEq for UserID {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl std::cmp::Eq for UserID{}

impl std::cmp::PartialOrd for UserID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[derive(IntoStaticStr)]
pub enum AccessScopes {
    #[strum(to_string = "AD")]
    Admin,
    #[strum(to_string = "UM")]
    UserManagement,
    #[strum(to_string = "RM")]
    RulesManagement,
    #[strum(to_string = "SM")]
    ScriptsManagement,
    #[strum(to_string = "ICM")]
    InterconnectsManagement,
    #[strum(to_string = "IOM")]
    IOServiceManagement,
    #[strum(to_string = "AM")]
    AddonManagement,
    #[strum(to_string = "WM")]
    WebUIManagement,
    #[strum(to_string = "BM")]
    BackupsManagement,
    #[strum(to_string = "CERTM")]
    CertificateManagement,
    #[strum(to_string = "CC")]
    CoreConfig,
}