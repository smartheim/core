//! # The command line configuration is defined in this module.

use std::net::{Ipv4Addr, IpAddr};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
pub struct Config {
    /// Comma separated list of IP addresses to bind to for inter-process communication and network services.
    /// Binds to 0.0.0.0 if not set.
    #[structopt(short, long, env = "OHX_NETWORK_INTERFACES")]
    pub network_interfaces: Vec<IpAddr>,

    /// Addons require a valid refresh token during startup.
    /// Create a new token via ohx-cli if you are starting an Addon manually.
    /// Core services do not necessarily require this, as they have access to the private key and can generate a valid token themselves.
    pub refresh_token: Option<String>
}


impl Config {
    pub fn new() -> Config {
        Config {
            network_interfaces: vec![],
            refresh_token: None
        }
    }
}