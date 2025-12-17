use serde::Serialize;

/// Represents the response received from an ARP request.
#[derive(Serialize, Debug, Clone)]
pub struct ArpResponse {
    /// The target IP address that was queried.
    pub target_ip: String,

    /// The MAC address corresponding to the target IP.
    pub target_mac: String,

    /// An optional alias for the target device.
    pub alias: Option<String>,
}
