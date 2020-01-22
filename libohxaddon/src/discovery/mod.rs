//! # Core Service Discovery
//! This is a simple UDP multicast based request/response discovery protocol.
//! There is unfortunately no modern mdns implementation for Rust yet.

mod resolver;

pub(crate) const DISCOVER_PORT: u16 = 5454;

use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use socket2::{Socket, Domain, Type, Protocol, SockAddr};
use tokio::sync::mpsc::{Sender, Receiver, channel};
use futures_util::future::{select, Either};

use log::{info, warn};
use std::pin::Pin;
use pin_project::pin_project;
use serde::{Serialize, Deserialize};
use semver::Version;
use std::time::{SystemTime, Duration};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use arc_swap::ArcSwap;
use hyper::Uri;
use arraystring::ArrayString;

use chrono::NaiveDateTime;
use std::ops::Add;

enum DiscoveryCommand {
    RequestService { service: ServiceResolveRequest, resolver_tx: Sender<ResolveResult> },
    UnicastResponse { peer: SocketAddr, is_v6: bool },
    CheckTimeout { response_id: u64 },
    Exit,
}

/// This type acts as a command supplier to the [`Discovery`] type
/// and allows to issues resolve tasks or to shutdown the service.
#[derive(Clone)]
pub struct DiscoveryResolver {
    command: Sender<DiscoveryCommand>,
}

/// The service registry resolves a service based on information handed via this type.
/// This serves as future proof request builder, but is also used as the datatype that is send over the wire.
/// Changes need to be backwards compatible.
#[derive(Serialize, Deserialize)]
pub struct ServiceResolveRequest {
    /// The service name
    pub service_name: String,
    /// The service version
    pub min_version: semver::Version,
    /// The maximum accepted service version, non inclusive. Ie the version range is [min, max).
    /// This is usually not set, or set to the next semver incompatible version.
    pub max_version: Option<semver::Version>,
}

impl ServiceResolveRequest {
    pub fn with_version(service_name: String, min_version: semver::Version, max_version: Option<semver::Version>) -> Self {
        Self { service_name, min_version, max_version }
    }
    pub fn new(service_name: String) -> Self {
        Self { service_name, min_version: semver::Version::new(0, 0, 0), max_version: None }
    }
}

#[derive(Clone)]
pub struct ServiceRegistry {
    service_cache: Arc<Mutex<BTreeMap<String, ResolveResult>>>,
    resolver: DiscoveryResolver,
}

impl ServiceRegistry {
    pub fn new(resolver: DiscoveryResolver) -> Self {
        Self { service_cache: Arc::new(Mutex::new(Default::default())), resolver }
    }
    /// Resolve the given service. If the service has been resolved before, the entry
    /// of the cache is returned after a successful connection test.
    pub async fn try_resolve(&self, request: ServiceResolveRequest) -> io::Result<ResolveResult> {
        let mut resolve_result = {
            let data = self.service_cache.lock().expect("Mutex access to service cache");
            match data.get(&request.service_name) {
                None => ResolveResult::Unresolved,
                Some(service_entry) => {
                    match service_entry {
                        ResolveResult::Success(resolved_service) => {
                            // TODO Test connection, if not successful return ResolveResult::Unresolved
                            return Ok(ResolveResult::Success(resolved_service.clone()));
                        }
                        // If a failed resolve is older than x minutes, re-resolve service by setting the state
                        // to "Unresolved".
                        ResolveResult::Timeout { last_attempt } | ResolveResult::CapacityLimit { last_attempt } |
                        ResolveResult::VersionMismatch { last_attempt } if last_attempt.add(chrono::Duration::seconds(60 * 5)) < chrono::Utc::now().naive_local() => {
                            ResolveResult::Unresolved
                        }
                        _ => service_entry.clone()
                    }
                }
            }
        };
        let service_name = request.service_name.clone();
        if let ResolveResult::Unresolved = resolve_result {
            resolve_result = self.resolver.clone().resolve(request).await?;
        }
        let mut data = self.service_cache.lock().expect("Mutex access to service cache");
        data.insert(service_name, resolve_result.clone());
        Ok(resolve_result)
    }
}

/// The result type of a service discovery.
/// A service has either been resolved or failed to resolve with some details attached.
#[derive(Clone)]
pub enum ResolveResult {
    /// Service has been resolved
    Success(ResolvedService),
    /// The service resolver ran into a timeout.
    /// This also happens if the service can be resolved but no gRPC connection can be established.
    Timeout { last_attempt: NaiveDateTime },
    /// The service resolver does not accept any more concurrent requests
    CapacityLimit { last_attempt: NaiveDateTime },
    /// The requested version mismatches the found version.
    /// The last attempt is stored, so that after a while a another attempt to resolve the service happens.
    /// The correct version might have been started in the mean time.
    VersionMismatch { last_attempt: NaiveDateTime },
    /// Internal state. You will never get this back from the discovery service or service registry
    Unresolved,
}

#[derive(Clone)]
pub struct ResolvedService {
    /// A service has been resolved because of a specific request/response id.
    /// A response_id is also the unix time in milliseconds that the service was requested to be resolved.
    pub response_id: u64,
    /// The service name
    pub service_name: String,
    /// The service version
    pub version: semver::Version,
    /// All endpoints (IP:port) of the peer service. Can be casted to SocketAddr.
    pub addresses: Vec<String>,
    /// A gRPC connection channel to one of the given IP addresses.
    pub rpc: tonic::transport::Channel,
}

pub type ResolvedServiceRef = Arc<ResolvedService>;
pub type LastResolvedService = ArcSwap<ResolvedService>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use env_logger::{DEFAULT_FILTER_ENV, Env, TimestampPrecision};
    use crate::discovery::resolver::Discovery;
    use std::net::ToSocketAddrs;

    #[tokio::test]
    async fn test_request_response() {
        let mut builder = env_logger::Builder::from_env(Env::new().filter_or(DEFAULT_FILTER_ENV, "info"));
        builder
            .format_timestamp(Some(TimestampPrecision::Seconds))
            .format_module_path(false)
            .init();

        let mut discovery1 = Discovery::new("own_service".to_owned(), Version::new(1, 0, 1), vec![SocketAddr::from(([127, 0, 0, 2], 1443))]);
        let mut sender1 = discovery1.revolver();
        let discovery1_fut = tokio::spawn(async move {
            if let Err(e) = discovery1.run().await {
                warn!("Discovery publisher failed: {}", e);
            }
        });

        let mut discovery2 = Discovery::new("other_service".to_owned(), Version::new(2, 0, 1), vec![SocketAddr::from(([128, 0, 0, 3], 2443))]);
        let mut sender2 = discovery2.revolver();
        let discovery2_fut = tokio::spawn(async move {
            if let Err(e) = discovery2.run().await {
                warn!("Discovery publisher failed: {}", e);
            }
        });


        use tokio::time::timeout;

        let r = timeout(Duration::from_secs(3),
                        sender1.resolve(ServiceResolveRequest::new("other_service".to_owned()))).await;
        let r = r.unwrap(); // Unwrap timeout
        let r = match r.unwrap() { // Unwrap Result
            ResolveResult::Success(r) => r,
            _ => panic!("Failed to resolve service")
        };
        assert_eq!(r.version, Version::new(2, 0, 1));
        assert_eq!(&r.service_name, "other_service");
        let addr : SocketAddr= r.addresses.get(0).unwrap().parse().ok().unwrap();
        assert!(addr.ip() == Ipv4Addr::new(128, 0, 0, 3));

        // Shutdown
        sender1.exit().await.unwrap();
        sender2.exit().await.unwrap();
        discovery1_fut.await.unwrap();
        discovery2_fut.await.unwrap();
    }
}