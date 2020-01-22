pub mod notification_channel_sse;
pub mod notification_channel_registry;
pub mod service;

/// Long running operations return a StatusEmitter.
/// A status emitter can either be awaited and the final return value evaluated (for tests for example),
/// or pushed into the notification service.
pub struct StatusEmitter {
    inner: tokio::sync::watch::Receiver<()>,
    abort: Option<std::sync::mpsc::Sender<()>>,
}

impl Drop for StatusEmitter {
    fn drop(&mut self) {
        if let Some(sender) = self.abort.take() {
            let _ = sender.send(());
        }
    }
}
