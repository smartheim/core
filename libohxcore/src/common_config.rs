//! # The command line configuration is defined in this module.

use std::net::IpAddr;
use std::path::PathBuf;
use structopt::StructOpt;

const ROOT_DIR_NAME: &'static str = "ohx_root_dir";

#[derive(StructOpt, Debug, Clone)]
pub struct Config {
    /// The ohx root directory.
    /// Core services expect a "backups", "config", "interconnects", "certs", "webui", "rules" and "scripts" sub-directory.
    #[structopt(parse(from_os_str), short, long, env = "OHX_ROOT_DIRECTORY")]
    pub root_directory: Option<PathBuf>,

    /// The certificate and encryption keys storage directory. The default is ohx_root_directory/certs.
    /// The directory will be watched for changed files.
    #[structopt(parse(from_os_str), long, env = "OHX_CERTS_DIRECTORY")]
    pub certs_directory: Option<PathBuf>,

    /// The translations file directory. A translation file is a valid [Fluent](https://projectfluent.org/) file.
    /// Files are expected to have the language ID (according to https://unicode.org/reports/tr35/tr35.html#Unicode_language_identifier)
    /// as base file name.
    /// For example, "en-US.tr" (American English)
    #[structopt(parse(from_os_str), long, env = "OHX_I18N_DIRECTORY")]
    pub i18n_directory: Option<PathBuf>,

    /// Comma separated list of IP addresses to bind to for inter-process communication and network services.
    /// Binds to 0.0.0.0 if not set.
    #[structopt(short, long, env = "OHX_NETWORK_INTERFACES")]
    pub network_interfaces: Vec<IpAddr>,

    /// The address of the influxDB instance.
    #[structopt(long, env = "OHX_INFLUX_DB")]
    pub influx_addr: Option<IpAddr>,

    /// Comma separated list of addon registry urls. An addon registry must adhere to the Docker Registry APIv2.
    #[structopt(short, long, env = "OHX_ADDON_REGISTRIES")]
    pub addon_registries: Vec<String>,

    /// Tells OHX core services that they run in a container.
    #[structopt(long, env = "OHX_CONTAINER_MODE")]
    pub container_mode: bool,

    /// OHX will terminate if the root_directory does not exist yet.
    /// Set this option to create the root directory and sub-directories instead.
    #[structopt(long, env = "OHX_CREATE_ROOT")]
    pub create_root: bool,
}


impl Config {
    pub fn new() -> Config {
        Config {
            create_root: false,
            root_directory: None,
            certs_directory: None,
            i18n_directory: None,
            network_interfaces: vec![],
            influx_addr: None,
            addon_registries: vec![],
            container_mode: false,
        }
    }
    pub fn get_root_directory(&self) -> PathBuf {
        self.root_directory.clone().unwrap_or(std::env::current_dir().expect("Current dir to work").join(ROOT_DIR_NAME))
    }
    pub fn get_service_config_directory(&self, service_name: &str) -> Result<PathBuf,std::io::Error> {
        let p =self.get_root_directory().join("config").join(service_name);
        std::fs::create_dir_all(&p)?;
        Ok(p)
    }
    pub fn get_certs_directory(&self) -> PathBuf {
        self.certs_directory.clone().unwrap_or(self.get_root_directory().join("certs"))
    }
    pub fn get_i18n_directory(&self) -> PathBuf {
        self.i18n_directory.clone().unwrap_or(self.get_root_directory().join("i18n"))
    }
}