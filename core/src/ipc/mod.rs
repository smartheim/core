//! # Interprocess communication: Core services and Core to Addons
//! gRPC over http2 is used for Core <--> Addons communication and via unix sockets for Core <-->Core communication.
//!
//! WebUIs use the http+json API and server-send-events defined in the http module.