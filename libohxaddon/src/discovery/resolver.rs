//! # Core Service Discovery
//! This is a simple UDP multicast based request/response discovery protocol.
//! There is unfortunately no modern mdns implementation for Rust yet.

const RECEIVE_BUFFER_SIZE: usize = 1024;
const DISCOVER_PORT: u16 = 5454;
const CONCURRENT_RESOLVERS: usize = 10;
const RESOLVER_TIMEOUT: Duration = Duration::from_secs(2);

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
use std::sync::Arc;
use arc_swap::ArcSwap;
use hyper::Uri;
use arraystring::{ArrayString, typenum::*};
use super::DiscoveryResolver;
use crate::discovery::{DiscoveryCommand, ResolveResult, ResolvedService, ServiceResolveRequest};
use serde_json::Error;

/// The discovery service allows to resolve other OHX core services.
/// A service might return multiple IP addresses.
///
/// You usually do not want to use the Discovery type directly, but the [`ServiceRegistry`]
/// instead, which caches performed discoveries and serialize them (to not concurrently request
/// for the same service during startup).
///
/// ## Usage details
///
/// There should always be at most one discovery service per process
/// and run() should also only be called once per discovery service instance.
///
/// One reason to restart the discovery service is a changed network topology
/// and changed interface IP addresses.
#[pin_project]
pub struct Discovery {
    response_packet: ServiceResolverPacket,
    command_rx: Option<Receiver<DiscoveryCommand>>,
    resolver: DiscoveryResolver,
}

type IDType = ArrayString<U5>;
type ChallengeType = [char;32];

#[derive(Serialize, Deserialize)]
struct ServiceResolverPacket {
    /// ID must be OHXr1 (request packets) or OHXo1 (responses)
    id: IDType,
    #[serde(flatten)]
    data: ServiceResolverPacketType,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum ServiceResolverPacketType {
    Request {
        /// A request id. Responses to this request will have a matching response_id.
        /// The chosen id should be monotonic increasing, the unix timestamp in milliseconds is recommended.
        challenge: ChallengeType,
        /// The actual request
        request: ServiceResolveRequest,
    },
    RequestResponse {
        /// When a service with a request_id is requested, the response will be a unicast response with the same response_id.
        /// This is not a unique ID like with a tcp sequence number.
        /// Multiple versions of the same service may be operational at the same time and will respond with
        /// the same response_id.
        response_id: u64,
        service_name: String,
        version: semver::Version,
        service_addresses: Vec<String>,
    },
}

impl DiscoveryResolver {
    /// Resolve another service on the network
    pub async fn resolve(&mut self, service: ServiceResolveRequest) -> io::Result<ResolveResult> {
        let (resolver_tx, mut resolver_rx) = channel(1);

        self.command.send(DiscoveryCommand::RequestService { service, resolver_tx }).await
            .map_err(|_e| io::Error::new(io::ErrorKind::BrokenPipe, "Discovery command channel broken"))?;
        let service_result: ResolveResult = resolver_rx.recv().await
            .ok_or(io::Error::new(io::ErrorKind::ConnectionAborted, "Discovery Resolver unexpectedly closed"))?;
        Ok(service_result)
    }
    /// Exit the discovery service. The associated future will return after this call and any further
    /// use of this object will result in broken channel errors.
    pub async fn exit(&mut self) -> io::Result<()> {
        self.command.send(DiscoveryCommand::Exit).await
            .map_err(|_e| io::Error::new(io::ErrorKind::BrokenPipe, "Discovery command channel broken"))?;
        Ok(())
    }

    /// Issues a command to the discovery service resolved to check if the given expected response timed out.
    async fn check_for_timeout(&mut self, response_id: u64) -> io::Result<()> {
        self.command.send(DiscoveryCommand::CheckTimeout { response_id }).await
            .map_err(|_e| io::Error::new(io::ErrorKind::BrokenPipe, "Discovery command channel broken"))?;
        Ok(())
    }
}

impl Discovery {
    fn v4(port: u16) -> io::Result<UdpSocket> {
        let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
        let addr: SockAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port).into();
        socket.set_reuse_address(true)?;
        socket.set_reuse_port(true)?;
        socket.bind(&addr)?;
        socket.set_multicast_loop_v4(true)?;
        let addr = Ipv4Addr::new(224, 0, 0, 251);
        socket.join_multicast_v4(&addr, &Ipv4Addr::new(0, 0, 0, 0))?;
        Ok(socket.into_udp_socket())
    }

    fn v4_multicast_dest(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(224, 0, 0, 251)), port)
    }

    fn v6_multicast_dest(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0xfb)), port)
    }

    fn v6(port: u16) -> io::Result<UdpSocket> {
        let socket = Socket::new(Domain::ipv6(), Type::dgram(), Some(Protocol::udp()))?;
        let addr: SockAddr = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), port).into();
        socket.set_reuse_address(true)?;
        socket.set_reuse_port(true)?;
        socket.set_multicast_loop_v6(true)?;
        socket.set_only_v6(true)?;
        socket.bind(&addr)?;
        let addr = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0xfb);
        socket.join_multicast_v6(&addr, 0)?;
        Ok(socket.into_udp_socket())
    }

    /// Creates a challenge. A challenge is a 32 byte random string of latin characters and numbers (A-Za-z0-9)
    /// The response must contain the peers public key, the challenge and a MAC (message authentication code)
    fn make_challenge() -> ChallengeType {

    }

    /// Create a new service resolver instance. A service resolver also responds to peer resolver requests,
    /// therefore the service name, version and own addresses have to be given as parameters.
    pub fn new(service_name: String, service_version: semver::Version, service_addresses: Vec<SocketAddr>) -> Self {
        let (command_tx, command_rx) = channel(1);

        Discovery {
            response_packet: ServiceResolverPacket {
                id: IDType::from_chars("OHXo1".chars()),
                data: ServiceResolverPacketType::RequestResponse {
                    response_id: 0,
                    service_name,
                    version: service_version,
                    service_addresses: service_addresses.into_iter().map(|v| v.to_string()).collect(),
                },
            },
            command_rx: Some(command_rx),
            resolver: DiscoveryResolver { command: command_tx },
        }
    }

    /// Returns a service resolver.
    ///
    /// The resolver will not be usable (return IO errors) if this discovery service wasn't started before with run().
    pub fn revolver(&self) -> DiscoveryResolver {
        self.resolver.clone()
    }

    pub fn service_name(&self) -> &str {
        if let ServiceResolverPacketType::RequestResponse { response_id: _, service_name, version: _, service_addresses: _ } = &self.response_packet.data {
            &service_name
        } else { panic!("Discovery owned response packet is not a response packet anymore!"); }
    }

    /// Start this service discovery service. There should always be at most one discovery service
    /// and this method should also only be called once per discovery service instance.
    pub async fn run(&mut self) -> Result<(), std::io::Error> {
        // If run() has been called before, return early. Extract the receiver part of the command channel
        let mut command_rx = match self.command_rx.take() {
            Some(v) => v,
            None => return Ok(())
        };

        use tokio::net::UdpSocket;
        let socket_v4 = UdpSocket::from_std(Discovery::v4(DISCOVER_PORT)?)?;
        let socket_v6 = UdpSocket::from_std(Discovery::v6(DISCOVER_PORT)?)?;
        info!("Discovery listening on {:?} and {:?}", socket_v4.local_addr(), socket_v6.local_addr());

        let mut active_resolvers = BTreeMap::<u64, (Sender<ResolveResult>, ServiceResolveRequest)>::new();

        let (mut socket_receiver_v4, mut socket_sender_v4) = socket_v4.split();
        let (mut socket_receiver_v6, mut socket_sender_v6) = socket_v6.split();

        let mut buffer_v4: [u8; RECEIVE_BUFFER_SIZE] = [0; RECEIVE_BUFFER_SIZE];
        let mut buffer_v6: [u8; RECEIVE_BUFFER_SIZE] = [0; RECEIVE_BUFFER_SIZE];

        let (mut request_packet, mut receive_response_packet) = create_packets_upfront();

        loop {
            // Either the socket receives a new message or the channel resolved with a command
            let command: DiscoveryCommand = {
                // Receive packet. Excess bytes are discarded, therefore the returned packet length must be validated.
                let mut socket_receive_v4_fut = socket_receiver_v4.recv_from(&mut buffer_v4);
                let mut socket_receive_v6_fut = socket_receiver_v6.recv_from(&mut buffer_v6);

                let mut socket_receive_fut = select(unsafe { Pin::new_unchecked(&mut socket_receive_v4_fut) },
                                                    unsafe { Pin::new_unchecked(&mut socket_receive_v6_fut) });

                let mut command_receiver_fut = command_rx.recv();
                match select(unsafe { Pin::new_unchecked(&mut socket_receive_fut) },
                             unsafe { Pin::new_unchecked(&mut command_receiver_fut) }).await {
                    Either::Left((socket_result, _a)) => {
                        let ((size, peer), is_v6) = match socket_result {
                            Either::Left((socket_result, _a)) => (socket_result?, false),
                            Either::Right((socket_result, _b)) => (socket_result?, true)
                        };
                        // Only accept discovery packets of size <= 1kb
                        if size > RECEIVE_BUFFER_SIZE {
                            continue;
                        } else {
                            DiscoveryCommand::UnicastResponse { peer, is_v6 }
                        }
                    }
                    Either::Right((command, _b)) => {
                        match command {
                            Some(command) => command,
                            None => return Ok(())
                        }
                    }
                }
            };

            match command {
                DiscoveryCommand::Exit => {
                    if let ServiceResolverPacketType::RequestResponse { response_id: _, service_name, version: _, service_addresses: _ } = &self.response_packet.data {
                        info!("Stopping discovery/publisher of service {}", &service_name);
                    }
                    return Ok(());
                }
                DiscoveryCommand::CheckTimeout { response_id } => {
                    if let Some((mut resolver_tx_channel, _)) = active_resolvers.remove(&response_id) {
                        let _ = resolver_tx_channel.send(ResolveResult::Timeout { last_attempt: chrono::Utc::now().naive_local() }).await;
                    }
                }
                DiscoveryCommand::RequestService { service, mut resolver_tx } => {
                    if active_resolvers.len() > CONCURRENT_RESOLVERS {
                        let _ = resolver_tx.send(ResolveResult::CapacityLimit { last_attempt: chrono::Utc::now().naive_local() }).await;
                    } else {
                        info!("Request service {} ({})", &service.service_name, &service.min_version);
                        // Prepare outgoing packet
                        let response_id = Discovery::make_challenge();
                        if let ServiceResolverPacketType::Request { ref mut request_id, ref mut request } = request_packet.data {
                            *request_id = response_id;
                            request.service_name.clear();
                            request.service_name += &service.service_name;
                            request.min_version = service.min_version.clone();
                            request.max_version = service.max_version.clone();
                        }

                        // Store the callback channel and the request itself
                        active_resolvers.insert(response_id, (resolver_tx, service));
                        // Spawn another async task to check for a timeout
                        let mut resolver = self.resolver.clone();
                        tokio::spawn(async move {
                            tokio::time::delay_for(RESOLVER_TIMEOUT).await;
                            let _ = resolver.check_for_timeout(response_id).await;
                        });


                        let ser_data = self.discovery_packet_ser(&request_packet, &mut buffer_v4[..]);
                        let _ = socket_sender_v4.send_to(ser_data, &Discovery::v4_multicast_dest(DISCOVER_PORT)).await?;
                        let _ = socket_sender_v6.send_to(ser_data, &Discovery::v6_multicast_dest(DISCOVER_PORT)).await?;
                    }
                }
                DiscoveryCommand::UnicastResponse { peer, is_v6 } => {
                    let buffer = match is_v6 {
                        true => &mut buffer_v6[..],
                        false => &mut buffer_v4[..]
                    };
                    if let Some(mut packet) = packet_inplace_deserialize(&mut receive_response_packet, &mut request_packet, buffer) {
                        match &packet.data {
                            ServiceResolverPacketType::Request { ref request_id, ref request } => {
                                let socket = match is_v6 {
                                    true => &mut socket_sender_v6,
                                    false => &mut socket_sender_v4
                                };
                                info!("Received service request for {} from {}", &request.service_name, &peer);
                                if request.service_name == self.service_name() {
                                    let ser_data = self.discovery_packet_ser(&mut receive_response_packet, &mut buffer[..]);
                                    let amt = socket.send_to(ser_data, &peer).await?;
                                    info!("Send discovery response to {}", peer);
                                }
                            }
                            ServiceResolverPacketType::RequestResponse { response_id, service_name, version, service_addresses } => {
                                if let Some((mut resolver_tx, resolve_request)) = active_resolvers.remove(&response_id) {
                                    info!("Received response on {}", &peer);
                                    let rpc = connect_rpc(&service_addresses).await;
                                    if let Some(rpc) = rpc {
                                        info!("RPC connection established {}", peer);
                                        if let Err(e) = resolver_tx.send(ResolveResult::Success(ResolvedService {
                                            service_name: service_name.clone(),
                                            version: version.clone(),
                                            addresses: service_addresses.clone(),
                                            response_id: *response_id,
                                            rpc,
                                        })).await {
                                            warn!("Service resolver channel broken: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    fn discovery_packet_ser<'a>(&self, packet: &ServiceResolverPacket, data: &'a mut [u8]) -> &'a [u8] {
        use std::io::Cursor;
        let mut c = Cursor::new(&mut data[..]);
        serde_json::to_writer(&mut c, &packet).expect("Serializer to work");
        let written = c.position() as usize;
        drop(c);
        &data[..written]
    }
}

/// The default gRPC channel builder uses a tcp keep alive of 60 seconds, and at most 2 concurrent connections
/// with TCP nodelay option enabled and using https.
fn grpc_channel_builder(address: &str) -> tonic::transport::Endpoint {
    let uri = Uri::builder().scheme("https").authority(address).build().expect("Uri::builder with fixed schema and IP addr");
    tonic::transport::Channel::builder(uri)
        .tcp_keepalive(Some(Duration::from_secs(60))).concurrency_limit(2).tcp_nodelay(true)
}

/// Try to establish a connection to one of the given IP:port addresses.
/// Returns a connected gRPC channel or None.
async fn connect_rpc(addresses: &Vec<String>) -> Option<tonic::transport::Channel> {
    for address in addresses {
        let rpc = grpc_channel_builder(address).connect().await;
        match rpc {
            Ok(rpc) => return Some(rpc),
            Err(e) => {
                eprint!("error {}", e);
                continue;
            }
        }
    }
    return None;
}

// Create all possible receive data types up front.
// Ideally this will prevent any further allocations after a few rounds.
#[inline]
fn create_packets_upfront() -> (ServiceResolverPacket, ServiceResolverPacket) {
    let mut request_packet = ServiceResolverPacket {
        id: IDType::from_chars("OHXr1".chars()),
        data: ServiceResolverPacketType::Request { request_id: 0, request: ServiceResolveRequest::new(String::new()) },
    };
    let mut receive_response_packet = ServiceResolverPacket {
        id: IDType::from_chars("OHXo1".chars()),
        data: ServiceResolverPacketType::RequestResponse { response_id: 0, service_name: String::new(), version: Version::new(0, 0, 0), service_addresses: Vec::new() },
    };
    (request_packet, receive_response_packet)
}

/// Deserializes in place from "data" into either "receive_response_packet" or "request_packet"
/// depending on the type of the received packet (a valid packet has an 'id' key with a value OHXr.. or OHXo..).
///
/// Returns a reference to the selected buffer packet, where "data" has been deserialized to.
///
/// Both packet arguments refer to pre-initialized and re-used types for each of the two enum states of a [`ServiceResolverPacket`],
/// to avoid string and vector heap allocations for successive packets.
/// If we use just one output packet for both enum states, we would implicitly drop() allocated strings/vectors
/// most of the time. The usual receive pattern is an alternation between the two enum states.
fn packet_inplace_deserialize<'a>(receive_response_packet: &'a mut ServiceResolverPacket, request_packet: &'a mut ServiceResolverPacket, data: &[u8]) -> Option<&'a mut ServiceResolverPacket> {
    let buffer = if let Some(pos) = twoway::find_bytes(&data[0..10], b"OHX") {
        // Search for OHX and let position point to the ascii character right after the literal
        let pos = pos + 3;
        if data.len() <= pos { return None; }
        match data[pos] {
            b'r' => request_packet,
            b'o' => receive_response_packet,
            _ => return None
        }
    } else { return None; };

    // Use the above discovery packet and deserialize in place. This avoids string and vector heap allocations
    // for successive packets.
    let mut de = serde_json::de::Deserializer::from_slice(data);
    match ServiceResolverPacket::deserialize_in_place(&mut de, buffer) {
        Ok(_) => { Some(buffer) }
        Err(e) => {
            eprint!("error {}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let (mut request_packet, mut receive_response_packet) = create_packets_upfront();
        let data = b"{\"id\":\"OHXr1\",\"type\":\"Request\",\"request_id\":2,\"request\":{\"service_name\":\"service\",\"min_version\":\"1.0.0\"}}";
        let result = packet_inplace_deserialize(&mut request_packet, &mut receive_response_packet, &data[..]);
        assert!(result.is_some());
        let result = result.unwrap();
        match &result.data {
            ServiceResolverPacketType::Request { request_id, request } => {
                assert_eq!(*request_id, 2);
                assert_eq!(&request.service_name, "service");
                assert_eq!(request.min_version, semver::Version::new(1, 0, 0));
            }
            ServiceResolverPacketType::RequestResponse { .. } => { panic!("Request expected") }
        }
    }
}