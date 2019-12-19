//! # Core Service Discovery
//! This is a simple UDP multicast based request/response discovery protocol.
//! There is unfortunately no modern mdns implementation for Rust yet.

const DISCOVER_PORT: u16 = 5454;

use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket, SocketAddrV4};
use socket2::{Socket, Domain, Type, Protocol, SockAddr};
use tokio::sync::mpsc::{Sender, Receiver, channel};
use futures_util::future::{select, Either};

use log::{info,warn};
use std::pin::Pin;
use pin_project::pin_project;
use serde::{Serialize, Deserialize};
use semver::Version;
use std::time::SystemTime;
use std::collections::BTreeMap;

enum DiscoveryCommand {
    RequestService { service_name: String },
    UnicastResponseV4 { peer: SocketAddr },
    UnicastResponseV6 { peer: SocketAddr },
    Exit,
}

/// The discovery service allows to resolve other OHX core services.
/// A service might return multiple
#[pin_project]
pub struct Discovery {
    own_service_name: String,
    own_version: semver::Version,
    own_addresses: Vec<SocketAddr>,
    command_rx: Receiver<DiscoveryCommand>,
    resolver_tx: Sender<ResolvedService>,
    socket_v4: tokio::net::UdpSocket,
    socket_v6: tokio::net::UdpSocket,
}

pub struct DiscoveryResolver {
    command: Sender<DiscoveryCommand>,
    resolve: Receiver<ResolvedService>,
}

impl DiscoveryResolver {
    /// Resolve another service on the network
    pub async fn resolve(&mut self, service_name: String) -> io::Result<ResolvedService> {
        self.command.send(DiscoveryCommand::RequestService { service_name: service_name.clone() }).await
            .map_err(|_e| io::Error::new(io::ErrorKind::BrokenPipe, "Discovery command channel broken"))?;
        loop {
            let resolved_service: ResolvedService = self.resolve.recv().await
                .ok_or(io::Error::new(io::ErrorKind::ConnectionAborted, "Discovery Resolver unexpectedly closed"))?;
            if resolved_service.service_name == service_name {
                return Ok(resolved_service);
            }
        }
    }
    /// Exit the discovery service. The associated future will return after this call and any further
    /// use of this object will result in broken channel errors.
    pub async fn exit(&mut self) -> io::Result<()> {
        self.command.send(DiscoveryCommand::Exit).await
            .map_err(|_e| io::Error::new(io::ErrorKind::BrokenPipe, "Discovery command channel broken"))?;
        Ok(())
    }
}

pub struct ResolvedService {
    pub service_name: String,
    pub version: semver::Version,
    pub addresses: Vec<SocketAddr>,
}

#[derive(Serialize, Deserialize)]
struct DiscoveryPacket {
    /// ID must be OHX
    id: String,
    service_name: String,
    version: semver::Version,
    own_addresses: Vec<SocketAddr>,
    /// When a service is requested, the response will be a unicast response with the same response_id
    response_id: u64,
    request_service: Option<String>,
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

    pub fn new(service_name: String, version: semver::Version, own_addresses: Vec<SocketAddr>) -> Result<(DiscoveryResolver, Self), std::io::Error> {
        use tokio::net::UdpSocket;
        let socket_v4 = UdpSocket::from_std(Discovery::v4(DISCOVER_PORT)?)?;
        let socket_v6 = UdpSocket::from_std(Discovery::v6(DISCOVER_PORT)?)?;
        let (sender, receiver) = channel(1);
        let (resolver_tx, resolver_rx) = channel(1);

        info!("Discovery listening on {:?} and {:?}", socket_v4.local_addr(), socket_v6.local_addr());

        Ok((DiscoveryResolver { command: sender, resolve: resolver_rx }, Discovery {
            socket_v4,
            socket_v6,
            command_rx: receiver,
            resolver_tx,
            own_addresses,
            own_service_name: service_name,
            own_version: version,
        }))
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        let Discovery {
            own_service_name,
            own_version,
            own_addresses,
            mut command_rx,
            mut resolver_tx,
            socket_v4,
            socket_v6,
        } = self;

        let (mut socket_receiver_v4, mut socket_sender_v4) = socket_v4.split();
        let (mut socket_receiver_v6, mut socket_sender_v6) = socket_v6.split();

        let mut requests_in_flight = BTreeMap::<String, u64>::new();
        const BUF_LEN: usize = 1024;
        let mut buf_v4: [u8; BUF_LEN] = [0; BUF_LEN];
        let mut buf_v6: [u8; BUF_LEN] = [0; BUF_LEN];
        let mut packet = DiscoveryPacket {
            id: "OHX".to_string(),
            response_id: 0,
            service_name: "".to_string(),
            version: Version::new(0, 0, 0),
            own_addresses: Vec::new(),
            request_service: None,
        };

        loop {
            // Either the socket receives a new message or the channel resolved with a command
            let command: DiscoveryCommand = {
                // Receive packet. Excess bytes are discarded, therefore the returned packet length must be validated.
                let mut socket_receive_v4_fut = socket_receiver_v4.recv_from(&mut buf_v4);
                let mut socket_receive_v6_fut = socket_receiver_v6.recv_from(&mut buf_v6);

                let mut socket_receive_fut = select(unsafe { Pin::new_unchecked(&mut socket_receive_v4_fut) },
                                                    unsafe { Pin::new_unchecked(&mut socket_receive_v6_fut) });

                let mut command_receiver_fut = command_rx.recv();
                match select(unsafe { Pin::new_unchecked(&mut socket_receive_fut) },
                             unsafe { Pin::new_unchecked(&mut command_receiver_fut) }).await {
                    Either::Left((socket_result, _a)) => {
                        let ((size, peer), is_ipv6) = match socket_result {
                            Either::Left((socket_result, _a)) => (socket_result?, false),
                            Either::Right((socket_result, _b)) => (socket_result?, true)
                        };
                        // Only accept discovery packets of size <= 1kb
                        if size > BUF_LEN {
                            continue;
                        } else {
                            if is_ipv6 {
                                DiscoveryCommand::UnicastResponseV6 { peer }
                            } else {
                                DiscoveryCommand::UnicastResponseV4 { peer }
                            }
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
                    info!("Stopping discovery/publisher of service {}", &own_service_name);
                    return Ok(());
                }
                DiscoveryCommand::RequestService { service_name } => {
                    info!("Request service {}", &service_name);

                    packet.response_id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
                    requests_in_flight.insert(service_name.clone(), packet.response_id);
                    packet.request_service = Some(service_name);
                    let ser_data = discovery_packet_ser(&mut packet, &mut buf_v4[..],&own_service_name,own_version.clone(),&own_addresses)?;
                    let _ = socket_sender_v4.send_to(ser_data, &Discovery::v4_multicast_dest(DISCOVER_PORT)).await?;
                    let _ = socket_sender_v6.send_to(ser_data, &Discovery::v6_multicast_dest(DISCOVER_PORT)).await?;
                }
                DiscoveryCommand::UnicastResponseV4 { peer } => {
                    if discovery_packet_deser(&mut packet, &buf_v4[..]) {
                        if packet.id != "OHX" { continue; }
                        if let Some(requested_service) = packet.request_service.take() {
                            info!("Discovery v4 @{}. Received service request {} {}", &own_service_name, &requested_service, &peer);
                            if requested_service == own_service_name {
                                let ser_data = discovery_packet_ser(&mut packet, &mut buf_v4[..],&own_service_name,own_version.clone(),&own_addresses)?;
                                let amt = socket_sender_v4.send_to(ser_data, &peer).await?;
                                info!("Echoed {} bytes to {}", amt, peer);
                            }
                        } else {
                            info!("Discovery v4 @{}. Received response on {} of {}", &own_service_name, &peer, &packet.service_name);
                            if let Err(e) = resolver_tx.send(ResolvedService {
                                service_name: packet.service_name.clone(),
                                version: packet.version.clone(),
                                addresses: packet.own_addresses.clone(),
                            }).await {
                                warn!("Service resolver channel broken: {}", e);
                            }
                        }
                    }
                }
                DiscoveryCommand::UnicastResponseV6 { peer } => {
                    if discovery_packet_deser(&mut packet, &buf_v6[..]) {
                        if packet.id != "OHX" { continue; }
                        if let Some(requested_service) = packet.request_service.take() {
                            info!("Discovery v6 @{}. Received service request {} {}", &own_service_name, &requested_service, &peer);
                            if requested_service == own_service_name {
                                let ser_data = discovery_packet_ser(&mut packet, &mut buf_v6[..],&own_service_name,own_version.clone(),&own_addresses)?;
                                let amt = socket_sender_v6.send_to(ser_data, &peer).await?;
                                println!("Echoed {} bytes to {}", amt, peer);
                            }
                        } else {
                            info!("Discovery v6 @{}. Received response on {} of {}", &own_service_name, &peer, &packet.service_name);
                            let _ = resolver_tx.send(ResolvedService {
                                service_name: packet.service_name.clone(),
                                version: packet.version.clone(),
                                addresses: packet.own_addresses.clone(),
                            }).await;
                        }
                    }
                }
            }
        }
    }
}


fn discovery_packet_ser<'a>(packet: &mut DiscoveryPacket, data: &'a mut [u8], own_service_name:&str, own_version:semver::Version,own_addresses:&[SocketAddr]) -> Result<&'a [u8], std::io::Error> {

    packet.service_name.clear();
    packet.service_name += own_service_name;
    packet.version = own_version.clone();
    packet.own_addresses.clear();
    packet.own_addresses.extend_from_slice(own_addresses);

    use std::io::Cursor;
    let mut c = Cursor::new(&mut data[..]);
    serde_json::to_writer(&mut c, &packet)?;
    let written = c.position() as usize;
    drop(c);
    Ok(&data[..written])
}

fn discovery_packet_deser(packet: &mut DiscoveryPacket, data: &[u8]) -> bool {
    // Use the above discovery packet and deserialize in place. This avoids string and vector heap allocations
    // for successive packets.
    let mut de = serde_json::de::Deserializer::from_slice(data);
    DiscoveryPacket::deserialize_in_place(&mut de, packet).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddrV4;
    use std::time::Duration;
    use env_logger::{DEFAULT_FILTER_ENV, Env, TimestampPrecision};

    #[tokio::test]
    async fn test_request_response() {
        let mut builder = env_logger::Builder::from_env(Env::new().filter_or(DEFAULT_FILTER_ENV, "info"));
        builder
            .format_timestamp(Some(TimestampPrecision::Seconds))
            .format_module_path(false)
            .init();

        let discovery = Discovery::new("own_service".to_owned(), Version::new(1, 0, 1), vec![SocketAddr::from(([127, 0, 0, 2], 1443))]);
        let (mut sender1, discovery1) = discovery.unwrap();
        let discovery1_fut = tokio::spawn(async move {
            if let Err(e)=  discovery1.run().await {
                warn!("Discovery publisher failed: {}", e);
            }
        });

        let discovery = Discovery::new("other_service".to_owned(), Version::new(2, 0, 1), vec![SocketAddr::from(([128, 0, 0, 3], 2443))]);
        let (mut sender2, discovery2) = discovery.unwrap();
        let discovery2_fut = tokio::spawn(async move {
            if let Err(e)=  discovery2.run().await {
                warn!("Discovery publisher failed: {}", e);
            }
        });

        use tokio::time::{timeout, delay_for};
//        delay_for(Duration::from_secs(3)).await;

        let r = timeout(Duration::from_secs(3), sender1.resolve("other_service".to_owned())).await;
        let r = r.unwrap();
        let r: ResolvedService = r.unwrap();
        assert_eq!(r.version, Version::new(2, 0, 1));
        assert_eq!(&r.service_name, "other_service");
        assert!(r.addresses.get(0).unwrap().ip() == Ipv4Addr::new(128, 0, 0, 3));

        // Shutdown
        sender1.exit().await.unwrap();
        sender2.exit().await.unwrap();
        discovery1_fut.await.unwrap();
        discovery2_fut.await.unwrap();
    }
}