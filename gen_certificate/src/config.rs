//! # The command line configuration is defined in this module.

use std::net::Ipv4Addr;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)] //
pub struct Config {
    /// Gateway of the captive portal WiFi network. This address will be added to the certificate.
    #[structopt(
    short,
    long = "portal-gateway",
    env = "PORTAL_GATEWAY"
    )]
    pub gateway: Option<Ipv4Addr>,

    /// The directory where the certificate should reside.
    #[structopt(parse(from_os_str), short, long, env = "CERT_DIR")]
    pub cert_dir: Option<PathBuf>,

    /// The directory where the certificate should reside.
    #[structopt(short, long, env = "NO_TIME_WAIT")]
    pub no_time_wait: bool,
}
