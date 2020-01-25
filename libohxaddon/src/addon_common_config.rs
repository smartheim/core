//! # The command line configuration is defined in this module.

use std::net::{IpAddr};
use structopt::StructOpt;
use std::path::PathBuf;

#[derive(StructOpt, Debug, Clone)]
pub struct Config {
    /// Comma separated list of IP addresses to bind to for inter-process communication and network services.
    /// Binds to 0.0.0.0 if not set.
    #[structopt(short, long, env = "OHX_NETWORK_INTERFACES")]
    pub network_interfaces: Vec<IpAddr>,

    /// The translations file directory. A translation file is a valid [Fluent](https://projectfluent.org/) file.
    /// Files are expected to have the language ID (according to https://unicode.org/reports/tr35/tr35.html#Unicode_language_identifier)
    /// as base file name.
    /// For example, "en-US.tr" (American English)
    #[structopt(parse(from_os_str), long, env = "OHX_I18N_DIRECTORY")]
    pub i18n_directory: Option<PathBuf>,

    /// Addons require a valid refresh token during startup.
    /// Create a new token via ohx-cli if you are starting an Addon manually.
    /// Core services do not necessarily require this, as they have access to the private key and can generate a valid token themselves.
    #[structopt(long, env = "OHX_REFRESH_TOKEN")]
    pub refresh_token: Option<String>
}


impl Config {
    pub fn new() -> Config {
        Config {
            network_interfaces: vec![],
            i18n_directory: None,
            refresh_token: None
        }
    }
    pub fn get_i18n_directory(&self) -> PathBuf {
        self.i18n_directory.clone().unwrap_or(self.get_root_directory().join("i18n"))
    }
}