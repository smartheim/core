use semver::Version;
use tokio::io::AsyncBufRead;
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_core::Stream;
use futures_core::task::{Context, Poll};
use std::pin::Pin;

// * List IoServiceInstances of Addons
// * List Things of Addons
// * Execute command on Addon
// * Register to property changes