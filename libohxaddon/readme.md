# OHX Addon Library

Use this crate to develop OHX Addons in the programming language Rust.
The defined interfaces are also used by OHX Core.
By keeping the version of this library up-to-date, your addon will also be compatible to the newest OHX core version.

## Internationalisation (i18n)

OHX Core services and Addons default to an "i18n" directory (see `addon_common_config.rs`) for translation files.
Use the `i18n` module and a cloned, owned `Translations` type to translate your strings via an ID and optional placeholder values.

## Notifications

OHX Core services and Addons can publish temporary or permanent-until-acknowledged notifications.
The `notification` module provides the `PublishNotification` type to build and publish a notification.

## Core services discovery

The initial connection to core services is established by the discovery service that runs as in instance
in each Addon.