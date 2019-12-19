//! # The core specific command line configuration is defined in this module.

use std::net::Ipv4Addr;
use std::path::PathBuf;
use structopt::StructOpt;

use libohxcore::common_config;

#[derive(StructOpt, Debug, Clone)]
pub struct Config {
    /// OHX will terminate if the root_directory does not exist yet.
    /// Set this option to create the root directory and sub-directories instead.
    pub create_root: bool,
    /// The interconnects storage directory. The default is ohx_root_directory/interconnects.
    /// The directory will be watched for changed files.
    #[structopt(parse(from_os_str), long, env = "OHX_INTERCONNECTS_DIRECTORY")]
    pub interconnects_directory: Option<PathBuf>,
    /// The webui storage directory. The default is ohx_root_directory/webui.
    /// The directory will be watched for changed files.
    #[structopt(parse(from_os_str), long, env = "OHX_WEBUI_DIRECTORY")]
    pub webui_directory: Option<PathBuf>,
    /// The configuration directory. The default is ohx_root_directory/config/ohx-ruleengine.
    /// The directory will be watched for changed files.
    #[structopt(parse(from_os_str), long, env = "OHX_CORE_CONFIG_DIRECTORY")]
    pub core_config_directory: Option<PathBuf>,
}


impl Config {
    pub fn new() -> Config {
        Config {
            create_root: false,
            interconnects_directory: None,
            webui_directory: None,
            core_config_directory: None,
        }
    }
    pub fn get_webui_directory(&self, common_config: common_config::Config) -> PathBuf {
        self.webui_directory.clone().unwrap_or(common_config.get_root_directory().join("webui"))
    }
    pub fn get_interconnects_directory(&self, common_config: common_config::Config) -> PathBuf {
        self.interconnects_directory.clone().unwrap_or(common_config.get_root_directory().join("interconnects"))
    }
    pub fn get_config_directory(&self, common_config: common_config::Config) -> PathBuf {
        self.core_config_directory.clone().unwrap_or(common_config.get_root_directory().join("config/ohx-core"))
    }
}