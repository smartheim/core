//! # The core specific command line configuration is defined in this module.

use std::net::Ipv4Addr;
use std::path::PathBuf;
use structopt::StructOpt;

use libohxcore::common_config;

#[derive(StructOpt, Debug, Clone)]
pub struct Config {
    /// The users storage directory. The default is ohx_root_directory/config/users.
    /// The directory will be watched for changed files.
    #[structopt(parse(from_os_str), long, env = "OHX_AUTH_USERS_DIRECTORY")]
    pub users_directory: Option<PathBuf>,
    /// The configuration directory. The default is ohx_root_directory/config/ohx-ruleengine.
    /// The directory will be watched for changed files.
    #[structopt(parse(from_os_str), long, env = "OHX_SERVE_CONFIG_DIRECTORY")]
    pub auth_config_directory: Option<PathBuf>,
    #[structopt(flatten)]
    pub(crate) common: common_config::Config
}


impl Config {
    pub fn new() -> Config {
        Config {
            users_directory: None,
            auth_config_directory: None,
            common: common_config::Config::new()
        }
    }
    pub fn get_users_directory(&self, common_config: common_config::Config) -> PathBuf {
        self.users_directory.clone().unwrap_or(common_config.get_root_directory().join("config/users"))
    }
    pub fn get_config_directory(&self, common_config: common_config::Config) -> PathBuf {
        self.auth_config_directory.clone().unwrap_or(common_config.get_root_directory().join("config/ohx-auth"))
    }
}