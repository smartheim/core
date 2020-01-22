//! # The rule engine specific command line configuration is defined in this module.

use std::net::Ipv4Addr;
use std::path::PathBuf;
use structopt::StructOpt;

use libohxcore::common_config;

#[derive(StructOpt, Debug, Clone)]
pub struct Config {
    /// The rules storage directory. The default is ohx_root_directory/rules.
    /// The directory will be watched for changed files.
    #[structopt(parse(from_os_str), long, env = "OHX_RULES_DIRECTORY")]
    pub rules_directory: Option<PathBuf>,
    /// The scripts storage directory. The default is ohx_root_directory/scripts.
    /// The directory will be watched for changed files.
    #[structopt(parse(from_os_str), long, env = "OHX_SCRIPTS_DIRECTORY")]
    pub scripts_directory: Option<PathBuf>,
    /// The configuration directory. The default is ohx_root_directory/config/ohx-ruleengine.
    /// The directory will be watched for changed files.
    #[structopt(parse(from_os_str), long, env = "OHX_RULEENGINE_CONFIG_DIRECTORY")]
    pub ruleengine_config_directory: Option<PathBuf>,
    #[structopt(flatten)]
    pub(crate) common: common_config::Config
}


impl Config {
    pub fn new() -> Config {
        Config {
            rules_directory: None,
            scripts_directory: None,
            ruleengine_config_directory: None,
            common: common_config::Config::new()
        }
    }
    pub fn get_rules_directory(&self, common_config: common_config::Config) -> PathBuf {
        self.rules_directory.clone().unwrap_or(common_config.get_root_directory().join("rules"))
    }
    pub fn get_scripts_directory(&self, common_config: common_config::Config) -> PathBuf {
        self.scripts_directory.clone().unwrap_or(common_config.get_root_directory().join("scripts"))
    }
    pub fn get_config_directory(&self, common_config: common_config::Config) -> PathBuf {
        self.ruleengine_config_directory.clone().unwrap_or(common_config.get_root_directory().join("config/ohx-ruleengine"))
    }
}