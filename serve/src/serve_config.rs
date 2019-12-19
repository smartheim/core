//! # The core specific command line configuration is defined in this module.

use std::net::Ipv4Addr;
use std::path::PathBuf;
use structopt::StructOpt;

use libohxcore::common_config;

#[derive(StructOpt, Debug, Clone)]
pub struct Config {
    /// The webui storage directory. The default is ohx_root_directory/webui.
    /// The directory will be watched for changed files.
    #[structopt(parse(from_os_str), long, env = "OHX_WEBUI_DIRECTORY")]
    pub webui_directory: Option<PathBuf>,
    /// The configuration directory. The default is ohx_root_directory/config/ohx-ruleengine.
    /// The directory will be watched for changed files.
    #[structopt(parse(from_os_str), long, env = "OHX_SERVE_CONFIG_DIRECTORY")]
    pub serve_config_directory: Option<PathBuf>,
}


impl Config {
    pub fn new() -> Config {
        Config {
            webui_directory: None,
            serve_config_directory: None,
        }
    }
    pub fn get_webui_directory(&self, common_config: common_config::Config) -> PathBuf {
        self.webui_directory.clone().unwrap_or(common_config.get_root_directory().join("webui"))
    }
    pub fn get_config_directory(&self, common_config: common_config::Config) -> PathBuf {
        self.serve_config_directory.clone().unwrap_or(common_config.get_root_directory().join("config/ohx-core"))
    }
}