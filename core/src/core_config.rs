//! # The core specific command line configuration is defined in this module.

use std::net::Ipv4Addr;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
pub struct Config {
    /// The ohx root directory
    #[structopt(parse(from_os_str), short, long, env = "ROOT_DIRECTORY")]
    pub root_directory: Option<PathBuf>,

    /// OHX will terminate if the root_directory does not exist yet.
    /// Set this option to create the root directory and sub-directories instead.
    pub create_root: bool,
}


impl Config {
    pub fn new() -> Config {
        Config {
            root_directory: None,
            create_root: false,
        }
    }
    pub fn get_root_directory(&self) -> PathBuf {
        self.root_directory.clone().unwrap_or(std::env::current_dir().expect("Current dir to work").join(ROOT_DIR_NAME))
    }
}