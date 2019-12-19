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

pub type UserID = String;
pub type AclTag = String;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Access for user {} denied. The rule is associated with {}", "user.display()", "owner.display()"))]
    AccessDenied { user: UserID, owner: UserID },
}

pub struct Access {
    owner: UserID,
    acl: Vec<AclTag>,
}
