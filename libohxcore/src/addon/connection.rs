
/// Represents an addon connection within OHX core services and is used by the Rule Engine
/// to communicate with Addon provided Rule Engine Modules.
///
/// A connection is also used to propagate Property changes (for IOService Addons) and
/// to push notifications.
///
/// A connection can be lost. No action is performed by OHX in that case, as it is the
/// responsibility of an Addon to keep the connection going.
/// A TCP keep alive of 1 minute allows Core and an Addon to discover a broken connection within that period.
pub struct AddonConnection {

}