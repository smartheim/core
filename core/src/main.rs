mod certificates;

use env_logger::{Env, TimestampPrecision, DEFAULT_FILTER_ENV};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use warp::{Filter, Rejection, http::Uri, Buf, hyper, hyper::Body, Reply, http::{header, HeaderMap}};
use log::info;
use serde::Serialize;

use libohx::common_config;
use std::sync::{Arc, Mutex};
use tokio::time::Duration;
use std::rc::Rc;

fn create_root_directory(dir: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(dir.join("backups"))?;
    std::fs::create_dir_all(dir.join("certs"))?;
    std::fs::create_dir_all(dir.join("config"))?;
    std::fs::create_dir_all(dir.join("interconnects"))?;
    std::fs::create_dir_all(dir.join("rules"))?;
    std::fs::create_dir_all(dir.join("scripts"))?;
    std::fs::create_dir_all(dir.join("webui"))?;
    Ok(())
}

pub struct HttpService {
    http_root: PathBuf,

}

pub struct ShutdownState {
    shutdown: bool
}

#[derive(Clone)]
pub struct RedirectEntry {
    /// Might be an IP (with port) or a domain
    target: Arc<String>,
}

impl RedirectEntry {
    pub fn new(target: &str) -> Self {
        RedirectEntry { target: Arc::new(target.to_string()) }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = env_logger::Builder::from_env(Env::new().filter_or(DEFAULT_FILTER_ENV, "info"));
    builder
        .format_timestamp(Some(TimestampPrecision::Seconds))
        .format_module_path(false)
        .init();

    let config: common_config::Config = common_config::Config::from_args();

    let path = config.get_root_directory();
    if !config.create_root && !path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "OHX Root directory does not exist. Consider using --create-root").into());
    }
    create_root_directory(&path)?;

    certificates::wait_until_known_time(false)?;

    // Create certificates
    let cert_dir = path.join("certs");
    certificates::check_gen_certificates(&cert_dir)?;

    let (restart_http, mut restart_http_rx) = tokio::sync::mpsc::channel(1);
    let (restart_http_watch_tx, mut restart_http_watch_rx) = tokio::sync::watch::channel(false);
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(1);
    let http_shutdown_marker = Arc::new(Mutex::new(ShutdownState { shutdown: false }));

    let mut restart_http_clone = restart_http.clone();
    // The shutdown task: It will inform all other parts via channels about the shutdown
    let http_shutdown_marker_clone = http_shutdown_marker.clone();
    tokio::spawn(async move {
        shutdown_rx.recv().await;
        http_shutdown_marker_clone.lock().unwrap().shutdown = true;
        restart_http_clone.send(()).await.unwrap();
    });
    // The http notify restart task. It notifies the current server instance about a restart request
    let http_shutdown_marker_clone = http_shutdown_marker.clone();
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

    // Ctrl+C task
    let mut shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        loop {
            let _ = tokio::signal::ctrl_c().await;
            info!("Ctrl+C: Shutting down");
            shutdown_tx_clone.send(()).await.unwrap();
        }
    });

//    let mut shutdown_tx_clone = shutdown_tx.clone();
//    tokio::spawn(async move {
//        loop {
//            let _ = tokio::time::delay_for(Duration::from_secs(3)).await;
//            info!("Timeout: Shutting down");
//            shutdown_tx_clone.send(()).await.unwrap();
//        }
//    });

    // Start certificate refresh task with graceful shutdown warp channel
    tokio::spawn(async {});

    // ArcSwap'able default ui config state
    // Injected for the root get -> redirect http page
    let http_root_path = path.clone();
    let http_shutdown_marker_clone = http_shutdown_marker.clone();
    let server_future = tokio::spawn(async move {
        loop {
            let mut restart_http_watch_rx_clone = restart_http_watch_rx.clone();
            let static_file_serve = index_filter()
                .or(warp::path("common").and(extract_request(RedirectEntry::new("192.168.1.3"))))
                .or(warp::path("general").and(extract_request(RedirectEntry::new("192.168.1.3"))))
                .or(warp::path("ui").and(warp::fs::dir(http_root_path.join("webui"))));

            // Start tls warp with graceful shutdown
            let (addr, server) = warp::serve(static_file_serve)
//        .tls(certificates::cert_filename(&cert_dir, certificates::FileFormat::PEM), certificates::key_filename(&cert_dir, certificates::FileFormat::PEM))
                .bind_with_graceful_shutdown(([0, 0, 0, 0], 8080), async move {
                    while let Some(value) = restart_http_watch_rx_clone.recv().await {
                        if value { break; }
                    }
                });

            info!("HTTP server running on {}", addr);
            server.await;
            info!("HTTP server stopped");
            if http_shutdown_marker_clone.clone().lock().unwrap().shutdown { break; }
        }
    });

    server_future.await?;

    Ok(())
}

pub fn index_filter() -> impl Filter<Extract=(impl warp::reply::Reply, ), Error=Rejection> + Clone {
    warp::path::end().map(|| warp::redirect(Uri::from_static("/ui/readme.md"))).map(|reply| {
        warp::reply::with_header(reply, "server", "warp")
    })
}

#[derive(Serialize)]
struct Resp {
    path: String,
    status: String,
    body_req: String,
    body_response: String,
    headers: String,
}

async fn request_proxied_service(method: warp::http::Method, path: warp::path::FullPath, headers: HeaderMap, body: warp::body::FullBody, redirect: RedirectEntry) -> Result<impl Reply, Rejection> {
    let path_tail = path.as_str();
//    let mut path = String::with_capacity(path_tail.len() + 1);
//    path += "/";
//    path += path_tail;
//    let path_tail = &path[..];
    let uri = warp::http::Uri::builder().scheme(warp::http::uri::Scheme::HTTP).authority(&redirect.target.as_ref()[..]).path_and_query(path_tail).build().unwrap();
    info!("uri request {}", &uri);
    let data = body.bytes().to_vec();
    let size = data.len().to_string();
    let mut req = warp::http::Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::from(data))
        .expect("request builder");
    *req.headers_mut() = headers;
    req.headers_mut().append(header::CONTENT_LENGTH, header::HeaderValue::from_str(&size).unwrap());
    //*req.method_mut() = method;
    //let json = serde_json::to_string(&req).unwrap();
    let mut connector = hyper::client::HttpConnector::new();
    connector.set_nodelay(true);
    let r = hyper::Client::builder().build(connector).request(req).await.unwrap();
    Ok(r)
//    let resp = Resp {
//        path: path.as_str().to_string(),
//        status: r.status().to_string(),
//        body_req: unsafe { std::str::from_utf8_unchecked(body.bytes()) }.to_owned(),
//        headers: r.headers().iter().fold(String::new(), |a, v| { a + " " + v.0.as_str() + " " + v.1.to_str().unwrap() }).to_string(),
//        body_response: unsafe { std::str::from_utf8_unchecked(hyper::body::to_bytes(r.into_body()).await.unwrap().bytes()) }.to_owned(),
//    };
    //Ok(warp::reply::json(&resp))
}

fn extract_request(redirect: RedirectEntry) -> impl Filter<Extract=(impl warp::reply::Reply, ), Error=Rejection>+Clone {
    let db = warp::any().map(move || -> RedirectEntry { redirect.clone() });
    warp::method()
        .and(warp::path::full())
        .and(warp::header::headers_cloned())
        .and(warp::body::concat())
        .and(db.clone())
        .and_then(request_proxied_service)
}