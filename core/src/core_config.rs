//! # The core specific command line configuration is defined in this module.

use std::net::Ipv4Addr;
use std::path::PathBuf;
use structopt::StructOpt;
use std::str::FromStr;

use libohxcore::common_config;

#[derive(StructOpt, Debug, Clone)]
pub enum LowMemoryPolicy {
    GraduallyRestartAddons
}

impl FromStr for LowMemoryPolicy {
    type Err = &'static str;
    fn from_str(policy: &str) -> Result<Self, Self::Err> {
        match policy {
            "GraduallyRestartAddons" => Ok(LowMemoryPolicy::GraduallyRestartAddons),
            _ => Err("Could not parse LowMemoryPolicy"),
        }
    }
}


#[derive(StructOpt, Debug, Clone)]
pub enum LowDiskSpacePolicy {
    StopAddons
}

impl FromStr for LowDiskSpacePolicy {
    type Err = &'static str;
    fn from_str(policy: &str) -> Result<Self, Self::Err> {
        match policy {
            "StopAddons" => Ok(LowDiskSpacePolicy::StopAddons),
            _ => Err("Could not parse LowDiskSpacePolicy"),
        }
    }
}

#[derive(StructOpt, Debug, Clone)]
pub struct Config {
    /// OHX will terminate if the root_directory does not exist yet.
    /// Set this option to create the root directory and sub-directories instead.
    pub create_root: bool,

    /// The interconnects storage directory. The default is ohx_root_directory/interconnects.
    /// The directory will be watched for changed files.
    #[structopt(parse(from_os_str), long, env = "OHX_INTERCONNECTS_DIRECTORY")]
    pub interconnects_directory: Option<PathBuf>,

    /// The configuration directory. The default is ohx_root_directory/config/ohx-ruleengine.
    /// The directory will be watched for changed files.
    #[structopt(parse(from_os_str), long, env = "OHX_CORE_CONFIG_DIRECTORY")]
    pub core_config_directory: Option<PathBuf>,

    /// Define how to react on memory pressure before the OS starts to kill random processes.
    /// The default policy restarts Addons with high memory usage as those may just have leaked
    /// memory over time and a restart would bring the system back to a good condition.
    #[structopt(long, default_value="GraduallyRestartAddons", env = "OHX_LOW_MEM_POLICY")]
    pub low_memory_policy: LowMemoryPolicy,

    /// Define how to react on low disk space. OHX will already implicitly purge logs and request
    /// Addons to clear cached files.
    ///
    /// Low disk space might not be solvable by OHX alone and requires user interaction at some point.
    /// OHX can only try to keep running as long as possible, therefore the default policy is to
    /// gradually stop Addons that consume a lot of disk space.
    #[structopt(long, default_value="StopAddons", env = "OHX_LOW_DISK_POLICY")]
    pub low_disk_space_policy: LowDiskSpacePolicy,

    /// Addons that consume more disk space than allowed as per Addon specification and permissions,
    /// will be stopped if in standalone mode. Set this flag to disable this behaviour.
    #[structopt(long, env = "OHX_DISABLE_QUOTA_ENFORCEMENT")]
    pub disable_quota_enforcement: bool,

    /// Addon Management happens via Docker or Podman. This is auto-detected during runtime and podman
    /// is preferred. Set this option to force using docker instead.
    #[structopt(long, env = "OHX_FORCE_DOCKER")]
    pub force_docker: bool
}


impl Config {
    pub fn new() -> Config {
        Config {
            create_root: false,
            interconnects_directory: None,
            core_config_directory: None,
            low_memory_policy:LowMemoryPolicy::GraduallyRestartAddons,
            low_disk_space_policy: LowDiskSpacePolicy::StopAddons,
            disable_quota_enforcement: false,
            force_docker: false
        }
    }
    pub fn get_interconnects_directory(&self, common_config: common_config::Config) -> PathBuf {
        self.interconnects_directory.clone().unwrap_or(common_config.get_root_directory().join("interconnects"))
    }
    pub fn get_config_directory(&self, common_config: common_config::Config) -> PathBuf {
        self.core_config_directory.clone().unwrap_or(common_config.get_root_directory().join("config/ohx-core"))
    }
}