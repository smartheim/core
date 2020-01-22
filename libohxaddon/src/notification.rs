use strum_macros::IntoStaticStr;
use std::collections::HashMap;
use tokio::sync::mpsc::{Sender, Receiver};

use crate::i18n::{Translations, FluentValue};
use std::borrow::Cow;
use std::sync::atomic::{AtomicUsize, Ordering};
use chrono::{Utc, DateTime};

#[derive(IntoStaticStr, Clone, Copy)]
pub enum NotificationCategory {
    DiskSpaceInsufficient,
    NoWriteAccess,
    ConfigurationInvalid,
    RestartImminent,
    Other,
}

#[derive(IntoStaticStr, Clone, Copy)]
pub enum NotificationUrgency {
    Error,
    Warning,
    Info,
}

/// A notification might be related to something.
/// Core services and Addons can provide a reference to the related entity so that user interfaces
/// can render an actual link.
#[derive(Clone)]
pub enum NotificationLink {
    None,
    AddonID(String),
    ThingID(String),
    ThingProperty(String),
}

#[derive(Clone, Copy)]
pub enum NotificationInteraction {
    None,
    RequiresConfirmation,
}

pub type NotificationTags = Vec<String>;

/// A notification is purely informational (for example "Device connected") or informs about required actions to take ("Press pairing button") or
/// unusual conditions that endanger the continuous running of the application ("Low on disk space").
///
/// A notification consists of a title and message and a unique id. It has an urgency and category as well as optional tags attached.
///
/// OHX will keep about 100 require-no-confirmation and all not-yet-confirmed confirmable notifications in memory for users to review.
/// Notifications will be purged from this backlog cache depending on their urgency and freshness.
#[derive(Clone)]
pub struct Notification {
    id: usize,
    component: Cow<'static, str>,
    cat: NotificationCategory,
    urgency: NotificationUrgency,
    tags: NotificationTags,
    link: NotificationLink,
    confirm: NotificationInteraction,
    title: String,
    message: String,
    issued: DateTime<Utc>,
}

impl Notification {
    pub fn new(id: usize, component: &'static str, urgency: NotificationUrgency, cat: NotificationCategory, title: String, message: String) -> Notification {
        Notification {
            id,
            component: Cow::Borrowed(component),
            link: NotificationLink::None,
            confirm: NotificationInteraction::None,
            cat,
            urgency,
            tags: Default::default(),
            title,
            message,
            issued: Utc::now(),
        }
    }
    pub fn with_tags(&mut self, tags: NotificationTags) {
        self.tags = tags;
    }
    pub fn with_link(&mut self, link: NotificationLink) {
        self.link = link;
    }
    /// Requires that this confirmation is confirmed by the user.
    /// Such a notification will re-appear again and again until the user has acknowledged it.
    pub fn confirm(&mut self) {
        self.confirm = NotificationInteraction::RequiresConfirmation;
    }
}

static GLOBAL_NOTIFICATION_ID: AtomicUsize = AtomicUsize::new(0);

pub struct PublishNotification {
    component: &'static str,
    cat: NotificationCategory,
    tags: Vec<String>,
    translations: Translations,
    sender: Sender<Notification>,
}

impl PublishNotification {
    pub fn new(component: &'static str, cat: NotificationCategory, tags: Vec<String>, translations: Translations, sender: Sender<Notification>) -> Self {
        Self {
            component,
            cat,
            tags,
            translations,
            sender,
        }
    }
    /// Build a notification, pre-filled with the arguments of this notification publisher
    pub fn build(&self, urgency: NotificationUrgency, title_translate_id: &'static str, message_translate_id: &'static str, args: HashMap<&str, FluentValue<'_>>) -> Notification {
        let id = GLOBAL_NOTIFICATION_ID.fetch_add(1, Ordering::SeqCst);
        Notification::new(id, self.component, urgency, self.cat, self.translations.tr(title_translate_id, Some(&args)).to_string(), self.translations.tr(message_translate_id, Some(&args)).to_string())
    }

    pub async fn publish(&self, n: Notification) -> Result<(), tokio::sync::mpsc::error::SendError<Notification>> {
        self.sender.clone().send(n).await
    }
}

pub mod rpc {

}