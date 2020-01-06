//! # Access control list: Interconnects and IOService configuration can optionally be locked to certain users via ACL tags
//! The rule engine will however always check an execution requests user token and set scopes to determine
//! if a user is allowed to run a rule. The ioservice manager and interconnect service will do the same.
//!
//! ACLs are just an additional method to allow more fine grained control.
//! It works by adding ACL tags to user accounts.
//! The next issued token (every 60 minutes or be logging in again) will contain the updated ACL tags.
//!
//! Certain Things and Rules might be restricted to certain ACl tags.

use snafu::Snafu;

/// A user has an ID that must be a valid filename (passes filename sanitizer).
pub struct UserID(String);
/// An ACL Tag is a string, that must not only consist of whitespaces
pub struct AclTag(String);

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Access for user {} denied. The rule is associated with {}", "user.display()", "owner.display()"))]
    AccessDenied { user: UserID, owner: UserID },
}

/// Rules, Scripts, Interconnections have an Access type attached.
/// By default only the owner and users with the correct AclTag can change such an object.
pub struct Access {
    owner: UserID,
    acl: Vec<AclTag>,
}

use strum_macros::IntoStaticStr;

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