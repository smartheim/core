//! # The configurable trait
//! A configurable type

use schemars::schema::Schema;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use std::path::{Path, PathBuf};
use std::io::{BufReader, BufWriter, Write};
use tokio::sync::mpsc::{Sender, Receiver};
use futures_util::future::{Abortable, AbortHandle, Aborted, AbortRegistration, Either};

use log::error;

pub use snafu::{ResultExt, Snafu};
use std::sync::Arc;
use std::collections::BTreeMap;
use sha2::{Sha256, Digest};
use sha2::digest::generic_array::{GenericArray, typenum::U64, typenum::U32};
use std::time::Duration;
use inotify::{Inotify, WatchMask, EventMask, EventStream, EventOwned};
use futures_util::StreamExt;
use std::pin::Pin;
use futures_util::stream::Next;
use std::str::FromStr;
use crate::schema_registry_trait::{SchemaRegistryTrait, SchemaRegistryCommand};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to open file for calculating checksum {}: {}", path.display(), source))]
    OpenFileForChecksum { source: std::io::Error, path: PathBuf },

    #[snafu(display("Failed to write default configuration to {}: {}", path.display(), source))]
    WriteDefaultConfig { source: std::io::Error, path: PathBuf },

    #[snafu(display("Unable to delete configuration file: {}", source))]
    DeleteConfigFile { source: std::io::Error },

    #[snafu(display("Serde error: {}", source))]
    SerdeError { source: serde_json::Error },

    #[snafu(display("Error happened while watching configuration directory {}: {}", path.display(), source))]
    ConfigDirWatcherError { source: std::io::Error, path: PathBuf },
}


pub type ConfigChangedResponse = Result<(), ()>;

pub enum ConfigChangedCommand {
    /// The original config file has been altered.
    /// The receiving party may find this file invalid and overwrite it with its current config instead.
    OriginalFileChanged(tokio::sync::oneshot::Sender<ConfigChangedResponse>),
    /// The configuration file with a "_" prefix has been found.
    /// The file must be moved to the original config filename after it has been validated and applied.
    NewFile(tokio::sync::oneshot::Sender<ConfigChangedResponse>, PathBuf),
    /// Config file has been deleted. The default configuration should be written now.
    FileDeleted(tokio::sync::oneshot::Sender<ConfigChangedResponse>),
}

pub enum ConfigWatcherCommand {
    StopConfigurationWatcher,
    ForcedReload { schema_name: String },
    /// Write the
    WriteChange { schema_name: String, config_serialized: String },
}

struct ConfigurationWatcherEntry {
    config_changed_sender: Sender<ConfigChangedCommand>,
    hash: GenericArray<u8, U32>,
}

impl ConfigurationWatcherEntry {
    pub fn new(config_changed_sender: Sender<ConfigChangedCommand>, hash: GenericArray<u8, U32>) -> Self {
        Self { config_changed_sender, hash }
    }
}

pub struct ConfigurationWatcher {
    configurations: BTreeMap<String, ConfigurationWatcherEntry>,
    config_path: PathBuf,
    watcher_cmd: Receiver<ConfigWatcherCommand>,
}

/// Compute sha256 checksum
fn chksum(path: &Path) -> Result<GenericArray<u8, U32>, Error> {
    let file = std::fs::File::open(&path).context(OpenFileForChecksum { path: path.clone() })?;
    let mut buffered_reader = BufReader::new(file);
    let mut sha256 = Sha256::new();
    std::io::copy(&mut buffered_reader, &mut sha256).context(OpenFileForChecksum { path: path.clone() })?;
    Ok(sha256.result())
}

impl ConfigurationWatcher {
    pub fn new(config_path: &Path) -> (Self, Sender<ConfigWatcherCommand>) {
        let (abort_handle, watcher_cmd) = tokio::sync::mpsc::channel::<ConfigWatcherCommand>(1);
        (Self {
            configurations: Default::default(),
            config_path: config_path.to_path_buf(),
            watcher_cmd,
        }, abort_handle)
    }
    /// Loads the configuration file if there is any based on the schema ie struct name.
    /// Loading will never fail, even if the file is malformed.
    ///
    /// If there is no file, the defaults will be serialized into a file.
    /// This may return an error (directory not writable, storage space insufficient, etc), this
    /// should abort the application during startup.
    ///
    /// Returns a tuple of (config_type, config_changed_validate_channel)
    pub fn register<T>(&mut self, config_changed_sender: Sender<ConfigChangedCommand>) -> Result<T, Error>
        where T: for<'de> Deserialize<'de> + Serialize + JsonSchema + Default {
        let schema_name = T::schema_name();

        let path = self.config_path.join(&schema_name);
        let me = if let Ok(file) = std::fs::File::open(&path) {
            let buffered_reader = BufReader::new(file);
            serde_json::from_reader(buffered_reader).context(SerdeError {})?
        } else {
            let file = std::fs::File::create(&path).context(WriteDefaultConfig { path: path.clone() })?;
            let buffered_writer = BufWriter::new(file);
            let me = T::default();
            serde_json::to_writer(buffered_writer, &me).context(SerdeError {})?;
            me
        };

        self.configurations.insert(schema_name, ConfigurationWatcherEntry::new(config_changed_sender, chksum(&path)?));
        Ok(me)
    }

    /// Start the configuration file watcher server.
    /// It will inform registered configuration change interested parties.
    ///
    /// # Panic
    /// If no observer has been registered
    pub async fn run(self) -> Result<(), Error> {
        use futures_util::future::select;

        let ConfigurationWatcher { mut configurations, config_path, mut watcher_cmd } = self;

        if configurations.is_empty() {
            panic!("No configuration files registered!");
        }

        let mut inotify = Inotify::init().context(ConfigDirWatcherError { path: config_path.clone() })?;

        inotify.add_watch(&config_path, WatchMask::CLOSE_WRITE | WatchMask::MOVED_TO | WatchMask::DELETE)
            .expect("Failed to add file watch");

        let mut buffer = Vec::with_capacity(1024);
        let mut stream = inotify.event_stream(&mut buffer).context(ConfigDirWatcherError { path: config_path.clone() })?;

        loop {
            let mut event = stream.next();
            let event = unsafe { Pin::new_unchecked(&mut event) };
            let mut watcher_cmd_fut = watcher_cmd.recv();
            let watcher_cmd_fut = unsafe { Pin::new_unchecked(&mut watcher_cmd_fut) };

            let r = select(event, watcher_cmd_fut).await;
            match r {
                Either::Left((streamed_event, _x)) => {
                    // If the stream is EOF, return from loop
                    let event = match streamed_event {
                        Some(Ok(v)) => v,
                        Some(Err(e)) => {
                            error!("Configuration file watch finished unexpectedly: {}", e);
                            break;
                        }
                        None => {
                            error!("Configuration file watch finished unexpectedly");
                            break;
                        }
                    };
                    let event: EventOwned = event;
                    if let Some(filename) = event.name {
                        if let Some(filename) = filename.to_str() {
                            let path = std::path::Path::new(filename);
                            let is_json = match path.extension() {
                                Some(v) => v == "json",
                                None => false
                            };
                            if !is_json { continue; }

                            let is_new = match path.file_name() {
                                Some(v) => v.to_str().expect("").starts_with("_"),
                                None => false
                            };
                            let schema_name = path
                                .file_stem().expect("A file stem to exist")
                                .to_str().expect("A str version of filename");
                            inform_about_changed_file(&mut configurations, path, schema_name, is_new, event.mask.contains(EventMask::DELETE)).await?;
                        }
                    }
                }
                Either::Right((watcher_cmd, _)) => {
                    let watcher_cmd: Option<ConfigWatcherCommand> = watcher_cmd;
                    match watcher_cmd {
                        Some(ConfigWatcherCommand::ForcedReload { schema_name }) => {
                            let file_name = config_path.with_file_name(&schema_name).with_extension("json");
                            inform_about_changed_file(&mut configurations, &file_name, &schema_name, false, false).await?;
                        }
                        Some(ConfigWatcherCommand::WriteChange { schema_name, config_serialized }) => {
                            let file_name = config_path.with_file_name(schema_name).with_extension("json");
                            let file = std::fs::File::open(&file_name).context(OpenFileForChecksum { path: file_name.clone() })?;
                            let mut buffered_writer = BufWriter::new(file);
                            buffered_writer.write(config_serialized.as_bytes()).context(OpenFileForChecksum { path: file_name.clone() })?;
                        }
                        Some(ConfigWatcherCommand::StopConfigurationWatcher) => { break; }
                        // The other end of the command channel has vanished -> cancel the file watcher
                        None => { break; }
                    }
                }
            };
        }
        Ok(())
    }
}

async fn inform_about_changed_file(configurations: &mut BTreeMap<String, ConfigurationWatcherEntry>, filename: &Path, schema_name: &str, is_new: bool, is_delete: bool) -> Result<(), Error> {
    let mut target_gone = false;

    if let Some(mut watcher_entry) = configurations.get_mut(schema_name) {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let (command, hash) = match (is_new, is_delete) {
            (true, false) => (ConfigChangedCommand::NewFile(sender, filename.to_path_buf()), chksum(&filename)?),
            (false, false) => (ConfigChangedCommand::OriginalFileChanged(sender), chksum(&filename)?),
            (true, true) => { // Ignore removed new files
                return Ok(());
            }
            (false, true) => (ConfigChangedCommand::FileDeleted(sender), GenericArray::default()),
        };

        if is_delete || hash != watcher_entry.hash {
            match watcher_entry.config_changed_sender.send(command).await {
                Err(_) => {
                    target_gone = true;
                }
                Ok(_) => {
                    // Await the file processing
                    if let Ok(r) = receiver.await {
                        let r: ConfigChangedResponse = r;
                        //TODO
                        // Update the hash
                        watcher_entry.hash = hash;
                    }
                    // Remove the file with _ prefix after processing
                    if !is_delete && is_new {
                        std::fs::remove_file(filename).context(DeleteConfigFile {})?;
                    }
                }
            }
        }
    }

    // The sender channel didn't work which means the receiving end has closed down.
    // Remove the registration.
    if target_gone {
        configurations.remove(schema_name);
    }
    Ok(())
}
