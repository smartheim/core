//! # The configurable trait
//! A configurable type

use schemars::schema::Schema;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use std::path::{Path, PathBuf};
use std::io::{BufReader, BufWriter};
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
    /// The configuration file with a .new suffix has been found.
    /// The file must be moved to the original config filename after it has been validated and applied.
    NewFile(tokio::sync::oneshot::Sender<ConfigChangedResponse>, PathBuf),
    /// Config file has been deleted. The default configuration should be written now.
    FileDeleted(tokio::sync::oneshot::Sender<ConfigChangedResponse>),
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
    abort_registration: Receiver<()>,
}

/// Compute checksum
fn chksum(path: &Path) -> Result<GenericArray<u8, U32>, Error> {
    let file = std::fs::File::open(&path).context(OpenFileForChecksum { path: path.clone() })?;
    let mut buffered_reader = BufReader::new(file);
    let mut sha256 = Sha256::new();
    std::io::copy(&mut buffered_reader, &mut sha256).context(OpenFileForChecksum { path: path.clone() })?;
    Ok(sha256.result())
}

impl ConfigurationWatcher {
    pub fn new(config_path: &Path) -> (Self, Sender<()>) {
        let (abort_handle, abort_registration) = tokio::sync::mpsc::channel(1);
        (Self {
            configurations: Default::default(),
            config_path: config_path.to_path_buf(),
            abort_registration,
        }, abort_handle)
    }
    /// Loads the configuration file if there is any based on the schema ie struct name.
    /// Loading will never fail, even if the file is malformed.
    /// If there is no file, the defaults will be serialized into a file.
    /// This may return an error (directory not writable, storage space insufficient, etc).
    pub fn register<T>(&mut self) -> Result<(T, Receiver<ConfigChangedCommand>), Error>
        where T: for<'de> Deserialize<'de> + Serialize + JsonSchema + Default {
        let path = self.config_path.join(T::schema_name());
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

        use tokio::sync::mpsc::channel;
        let (config_changed_sender, config_changed_receiver) = channel(1);

        self.configurations.insert(T::schema_name(), ConfigurationWatcherEntry::new(config_changed_sender, chksum(&path)?));
        Ok((me, config_changed_receiver))
    }

    /// Start the configuration file watcher server.
    /// It will inform registered configuration change interested parties.
    ///
    /// # Panic
    /// If no observer has been registered
    pub async fn run(self) -> Result<(), Error> {
        use futures_util::future::select;

        let ConfigurationWatcher { mut configurations, config_path, mut abort_registration } = self;

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
            let mut abort_fut = abort_registration.recv();
            let abort_fut = unsafe { Pin::new_unchecked(&mut abort_fut) };

            let r = select(event, abort_fut).await;
            match r {
                Either::Left((streamed_event, x)) => {
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
                            inform_about_changed_file(filename, &mut configurations, event.mask.contains(EventMask::DELETE)).await?;
                        }
                    }
                }
                Either::Right(_) => { break; }
            };
        }
        Ok(())
    }
}

async fn inform_about_changed_file(filename_orig: &str, configurations: &mut BTreeMap<String, ConfigurationWatcherEntry>, is_delete: bool) -> Result<(), Error> {
    let (is_new, filename) = {
        let is_new = filename_orig.ends_with(".new");
        (is_new, match is_new {
            true => &filename_orig[..filename_orig.len() - 4],
            false => filename_orig
        })
    };

    let mut target_gone = false;

    if let Some(mut v) = configurations.get_mut(filename) {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let (command, hash) = match (is_new, is_delete) {
            (true, false) => (ConfigChangedCommand::NewFile(sender, PathBuf::from_str(filename_orig).expect("Watch str to path")), chksum(Path::new(filename))?),
            (false, false) => (ConfigChangedCommand::OriginalFileChanged(sender), chksum(Path::new(filename))?),
            (true, true) => { // Ignore removed .new files
                return Ok(());
            }
            (false, true) => (ConfigChangedCommand::FileDeleted(sender),GenericArray::default()),
        };

        if is_delete || hash != v.hash {

            match v.config_changed_sender.send(command).await {
                Err(_) => {
                    target_gone = true;
                }
                Ok(_) => {
                    // Await the file processing
                    if let Ok(r) = receiver.await {
                        let r: ConfigChangedResponse = r;
                        // Update the hash
                        v.hash = hash;
                    }
                    // Remove the .new file after processing
                    if !is_delete && is_new {
                        std::fs::remove_file(filename_orig).context(DeleteConfigFile {})?;
                    }
                }
            }
        }
    }

    // The sender channel didn't work which means the receiving end has closed down.
    // Remove the registration.
    if target_gone {
        configurations.remove(filename);
    }
    Ok(())
}

pub type SchemaRegistryNotifier = Sender<SchemaRegistryCommand>;

pub enum SchemaRegistryCommand {
    Register(String),
    Unregister(String),
}

pub struct SchemaRegistryAutoUnregister {
    channel: Sender<SchemaRegistryCommand>,
    schema_name: String,
}

impl Drop for SchemaRegistryAutoUnregister {
    fn drop(&mut self) {
        let schema_name = self.schema_name.clone();
        let mut channel = self.channel.clone();
        tokio::spawn(async move {
            let _ = channel.send(SchemaRegistryCommand::Unregister(schema_name)).await;
        });
    }
}

pub trait Configurable: JsonSchema + Default {
    fn schema(&self) -> String {
        let schema = schemars::gen::SchemaGenerator::default().into_root_schema_for::<Self>();
        serde_json::to_string(&schema).expect("Valid jsonSchema annotations")
    }
}
