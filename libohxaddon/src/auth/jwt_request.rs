//! # Requests a new access token

use crate::users::{AccessScopes, UserID};
use crate::discovery::{DiscoveryResolver, ResolvedService};

pub struct JWTRequester {
    service_discovery: DiscoveryResolver,
    service: Option<ResolvedService>,
}

impl JWTRequester {
    /// Request a new access token from the ohx-auth service.
    /// If the service is not yet connected, it will be discovered and connected to first.
    pub fn request(scopes: Vec<AccessScopes>, user_id: UserID) {
        todo!()
    }
}