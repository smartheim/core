
/// Represents an (rpc) addon connection within OHX core services
///
/// A connection is used by
/// * the Rule Engine to communicate with Addon provided Rule Engine Modules
/// * to propagate Property changes (for IOService Addons) and
/// * to receive notifications.
///
/// A connection can be lost. No action is performed by OHX in that case, as it is the
/// responsibility of an Addon to keep the connection going.
pub struct AddonConnection {
    channel: tonic::transport::Channel
}