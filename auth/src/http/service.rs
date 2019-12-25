//! # The http service serves web-uis, and provides the configuration retrieval and manipulation REST API
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use warp::{Filter, Rejection, Buf, hyper, hyper::Body, Reply, http::{header, HeaderMap, Uri, self}};
use warp::reply::WithHeader;
use std::net::IpAddr;
use serde::{Deserialize, Serialize};

use log::{info, warn};

pub use snafu::{ResultExt, Snafu};
use arc_swap::{ArcSwap, ArcSwapOption};
use std::collections::BTreeMap;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Http Service is already running"))]
    HttpServerAlreadyRunning,

    #[snafu(display("UI Path does not exist: {}", "path.display()"))]
    UIPathDoesNotExist { path: PathBuf },

    #[snafu(display("URI Path Builder failed: {}", "source"))]
    UriPathBuilder { source: http::Error },
}

#[derive(Clone)]
pub struct RedirectEntryRaw {
    /// Might be an IP (with port) or a domain
    target: String,
    /// The http endpoint path for example "api" for /api.
    path: String,
    /// Entry ID: This is addon_id/path
    id: String,
}

type RedirectEntry = Arc<RedirectEntryRaw>;

impl RedirectEntryRaw {
    pub fn new(id: String, path: String, target: String) -> RedirectEntry {
        Arc::new(RedirectEntryRaw { id, path, target })
    }
    pub fn entry_id(addon_id: String, path: &str) -> String {
        addon_id + "/" + path
    }
}

/// A vector of redirect entries.
/// An entry consists of a target domain or IP address/port and a path like "/api" as redirect source.
///
/// The vector is wrapped with an ArcSwap. ArcSwap in contrast to Mutex allows interior mutability
/// via much faster pointer swaps without holding an actual operating system thread lock.
/// Please study the details of ArcSwap in their documentation.
///
/// This type is optimized for being read-accessed often (basically on every http request that didn't match
///with a higher priority route) with rare modifications.
type RedirectEntries = Arc<ArcSwap<RedirectEntriesVec>>;
type RedirectEntriesVec = Vec<RedirectEntry>;

type DefaultUiUri = Arc<ArcSwapOption<Uri>>;

/// All service status updates and notifications are pushed to the server-send-events endpoint on /ohx/v1/events.
///
pub struct EventPublisher {
    channel: mpsc::Sender<serde_json::Value>,
}

/// The http services serves an OAuth service.
pub struct HttpService {
    http_root: PathBuf,
    cert_key_pem_path: PathBuf,
    cert_public_pem_path: PathBuf,
    bind_addr: (IpAddr, u16),
    redirects: RedirectEntriesChanger,
    default_ui_id: DefaultUiUri,
    restart_http: mpsc::Sender<()>,
    restart_http_rx: Option<mpsc::Receiver<()>>,
    http_shutdown_marker: Arc<Mutex<ShutdownState>>,
}

#[derive(Default)]
pub struct RedirectEntriesChanger {
    /// A reference to the redirect entries. See the [`RedirectEntries`] type for more details.
    redirect_entries: RedirectEntries,
    /// The mutex ensures that concurrent changes via add / remove access the same vector instance.
    /// Without a locking mechanism, because ArcSwap is used, only one of the concurrent changes
    /// would make it.
    mtx: Arc<Mutex<bool>>,
    /// If the server has already been started, the restart channel sender will be used when `commit`
    /// is called to restart the service and apply the changes.
    restart_http: Option<mpsc::Sender<()>>,
}

impl RedirectEntriesChanger {
    /// Restart the http service to apply the added / removed proxies.
    /// This does nothing if the service is not yet running.
    pub async fn commit(&self) -> Result<(), mpsc::error::SendError<()>> {
        if let Some(mut sender) = self.restart_http.clone() {
            return sender.send(()).await;
        }
        Ok(())
    }
    pub fn add(&self, addon_id: String, target: String, path: String) {
        let _mtx = self.mtx.lock().expect("RedirectEntriesChanger lock");
        let mut vec = self.redirect_entries.load().as_ref().clone();

        let entry_id = RedirectEntryRaw::entry_id(addon_id, &path);
        if vec.iter().find(|e| e.id == entry_id).is_some() { return; }
        vec.push(RedirectEntryRaw::new(entry_id, path, target));
        self.redirect_entries.store(Arc::new(vec));
    }
    pub fn remove(&self, addon_id: String, path: String) {
        let _mtx = self.mtx.lock().expect("RedirectEntriesChanger lock");
        let mut vec = self.redirect_entries.load().as_ref().clone();

        let entry_id = addon_id + "/" + &path;
        if let Some(index) = vec.iter().position(|e| e.id == entry_id) {
            vec.remove(index);
        }
        self.redirect_entries.store(Arc::new(vec));
    }
}

struct ShutdownState {
    shutdown: bool
}

pub struct HttpServiceControl {
    http_shutdown_marker: Arc<Mutex<ShutdownState>>,
    restart_http: mpsc::Sender<()>,
    default_ui_id: DefaultUiUri,
    root_directory: PathBuf,
}

impl HttpServiceControl {
    /// Shutting down the server. This will make run() return.
    pub async fn shutdown(&self) {
        let http_shutdown_marker_clone = self.http_shutdown_marker.clone();
        http_shutdown_marker_clone.lock().unwrap().shutdown = true;
        let mut restart_http_clone = self.restart_http.clone();
        let _ = restart_http_clone.send(()).await;
    }

    /// Restart the server. This is necessary after routes have been added or removed.
    /// You usually do not need to call this manually.
    pub async fn restart(&self) {
        let mut restart_http_clone = self.restart_http.clone();
        let _ = restart_http_clone.send(()).await;
    }

    pub async fn set_default_ui(&self, ui_id: Option<String>) -> Result<(), Error> {
        if let Some(ui_id) = &ui_id {
            let path = self.root_directory.join(ui_id).join("index.html");
            if !path.exists() {
                return Err(Error::UIPathDoesNotExist { path });
            }
        }

        self.default_ui_id.store(match ui_id {
            Some(v) => {
                let uri = v + "/index.html";
                Some(Arc::new(Uri::builder().path_and_query(uri.as_str()).build().context(UriPathBuilder)?))
            }
            None => None
        });
        let mut restart_http_clone = self.restart_http.clone();
        let _ = restart_http_clone.send(()).await;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct UpdateOptions {
    mtime: Option<String>,
}

impl HttpService {
    /// Create a new instance of the warp based http service. Call `run` to start the service.
    pub fn new(http_root: PathBuf, cert_key_der_path: PathBuf, cert_public_der_path: PathBuf) -> Self {
        let (restart_http, restart_http_rx) = mpsc::channel(1);
        let http_shutdown_marker = Arc::new(Mutex::new(ShutdownState { shutdown: false }));

        HttpService {
            http_root,
            cert_key_pem_path: cert_key_der_path,
            cert_public_pem_path: cert_public_der_path,
            bind_addr: ([0, 0, 0, 0].into(), 8080),
            redirects: Default::default(),
            default_ui_id: Arc::new(ArcSwapOption::new(None)),
            restart_http,
            restart_http_rx: Some(restart_http_rx),
            http_shutdown_marker,
        }
    }

    pub fn redirect_entries(&self) -> RedirectEntriesChanger {
        RedirectEntriesChanger {
            redirect_entries: self.redirects.redirect_entries.clone(),
            mtx: self.redirects.mtx.clone(),
            restart_http: Some(self.restart_http.clone()),
        }
    }

    /// The returned type allows to restart and shutdown the http service.
    /// It also allows to set the default UI.
    pub fn control(&self) -> HttpServiceControl {
        HttpServiceControl {
            http_shutdown_marker: self.http_shutdown_marker.clone(),
            restart_http: self.restart_http.clone(),
            default_ui_id: self.default_ui_id.clone(),
            root_directory: self.http_root.clone(),
        }
    }

    /// Start the http server. This will spawn a few tasks onto the executor.
    /// The server itself runs in a loop, so that it can be restarted to apply new configuration.
    ///
    /// New configuration can be a changed ssl certificate or changed routes.
    /// A restart for the later is required, because route configuration for the dynamic parts (proxies etc)
    /// are cloned non-mutable Arc's.
    pub async fn run(&mut self) -> Result<(), Error> {
        let (restart_http_watch_tx, restart_http_watch_rx) = tokio::sync::watch::channel(false);

        let http_shutdown_marker_clone = self.http_shutdown_marker.clone();
        let mut restart_http_rx = match self.restart_http_rx.take() {
            Some(v) => v,
            None => return Err(Error::HttpServerAlreadyRunning)
        };

        // The http notify restart task. It notifies the current server instance about a restart request
        tokio::spawn(async move {
            loop {
                if let None = restart_http_rx.recv().await { break; }
                if http_shutdown_marker_clone.clone().lock().unwrap().shutdown { break; }
                // A watch channel receiver will always yield the last value on creation, so first send a true, then a false
                // to "reset".
                restart_http_watch_tx.broadcast(true).unwrap();
                restart_http_watch_tx.broadcast(false).unwrap();
            }
        });

        let http_root_path = self.http_root.clone();
        let http_shutdown_marker_clone = self.http_shutdown_marker.clone();
        let bind_addr = self.bind_addr;
        let redirect_entries = self.redirects.redirect_entries.clone();
        let default_ui_id = self.default_ui_id.clone();
        loop {
            let http_root_path = http_root_path.clone();
            let http_root_path_for_filter = http_root_path.clone();
            let http_root_path_filter = warp::any().map(move || http_root_path_for_filter.clone());
            let mut restart_http_watch_rx_clone = restart_http_watch_rx.clone();

            let route = make_root(self.redirects.redirect_entries.clone().load_full(),
                                  self.http_root.clone(), self.default_ui_id.clone());

            // Start tls warp with graceful shutdown
            let (addr, server) = warp::serve(route)
                .tls().key_path(&self.cert_key_pem_path).cert_path(&self.cert_public_pem_path)
                .bind_with_graceful_shutdown(bind_addr, async move {
                    while let Some(value) = restart_http_watch_rx_clone.recv().await {
                        if value { break; }
                    }
                });

            info!("HTTP server running on {}", addr);
            server.await;
            info!("HTTP server stopped");
            // Check if this is an actual shutdown, not just a restart request
            if http_shutdown_marker_clone.clone().lock().unwrap().shutdown { break; }
        }

        Ok(())
    }
}

/// Create the route for warp. The result is a boxed filer to reduce the compile time.
///
/// It also works with `impl Filter<Extract=(impl warp::Reply, ), Error=Rejection> + Clone`
/// but the resulting type is so complex that `#![type_length_limit="2313898"]` is required and compile time went up by 1 minute.
fn make_root(redirect_entries: Arc<RedirectEntriesVec>, http_root_path: PathBuf, default_ui_id: DefaultUiUri) -> warp::filters::BoxedFilter<(impl warp::Reply, )> {
    let redirect_entries = warp::any().map(move || redirect_entries.clone());
    let http_root_path_for_filter = http_root_path.clone();
    let http_root_path_filter = warp::any().map(move || http_root_path_for_filter.clone());

    warp::get()
        .and(warp::path("auth"))
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(warp::path::param::<String>())
        .and(http_root_path_filter.clone())
        .and_then(|module_id: String, schema_id: String, config_id: String, http_root: PathBuf| async move {
            let path = http_root.join(format!("config/{}/{}.{}.json", module_id, schema_id, config_id));
            if !path.exists() {
                return Err(warp::reject::not_found());
            }
            if let Err(e) = tokio::fs::remove_file(path).await {
                return Ok(warp::reply::with_status(warp::reply::html(NO_DEFAULT_UI_HTML), warp::http::StatusCode::INTERNAL_SERVER_ERROR));
            }
            Ok(warp::reply::with_status(warp::reply::html(""), warp::http::StatusCode::OK))
        })
        .or(warp::post()
            .and(warp::path("auth"))
            .and(warp::path::param::<String>())
            .and(warp::path::param::<String>())
            .and(warp::path::param::<String>())
            .and(warp::query::query::<UpdateOptions>())
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::concat())
            .and(http_root_path_filter.clone())
            .and_then(|module_id: String, schema_id: String, config_id: String, options: UpdateOptions, data: warp::body::FullBody, http_root: PathBuf| async move {
                //TODO
                Err::<String, Rejection>(warp::reject())
            })
        ).boxed()
}

const NO_DEFAULT_UI_HTML: &'static str = r#"<html>
    <head><title>OHX: No default UI</title></head>
    <body><h1>No default UI installed!</h1><p>Your installation seems incomplete.</p></body>
    </html>"#;

#[derive(Serialize)]
struct DirEntryForFilter {
    path: String,
    mtime: chrono::DateTime<chrono::Utc>,
}

/// Returns a json array containing the flattened directory content. For example:
/// ```json
/// ["subdir/file.abc", "another_file.def"]
/// ```
async fn directory_index_filter(path: warp::path::FullPath, http_root: PathBuf) -> Result<impl Reply, Rejection> {
    let path = http_root.join(&path.as_str()[1..]);
    if let Ok(mut entries) = tokio::fs::read_dir(path).await {
        let mut entries: tokio::fs::ReadDir = entries;
        let mut resp = Vec::<DirEntryForFilter>::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let (Ok(rel_path), Ok(metadata)) = (entry.path().strip_prefix(&http_root), entry.metadata().await) {
                let system_time = metadata.modified().unwrap_or(std::time::SystemTime::now());
                resp.push(DirEntryForFilter {
                    path: rel_path.to_str().unwrap_or_default().to_owned(),
                    mtime: system_time.into(),
                });
            } else {
                warn!("Couldn't get file type for {:?}", entry.path());
            }
        }
        return Ok(warp::reply::json(&resp));
    }
    Err(warp::reject())
}
