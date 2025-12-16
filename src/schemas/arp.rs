/// Represents the response received from an ARP request.
#[allow(dead_code)]
pub struct ArpResponse {
    /// The target IP address that was queried.
    pub target_ip: String,

    /// The MAC address corresponding to the target IP.
    pub target_mac: String,

    /// The sender IP address from which the ARP request was sent.
    pub sender_ip: String,

    /// The sender MAC address from which the ARP request was sent.
    pub sender_mac: String,

    /// The name of the network interface used to send the ARP request.
    pub interface_name: String,

    /// The timeout duration in seconds for waiting for an ARP response.
    pub timeout_secs: Option<f32>,
}
