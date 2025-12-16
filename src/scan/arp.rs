use std::net::{IpAddr, Ipv4Addr};
use std::time::SystemTime;

use pnet::datalink::{self, Channel, Config, NetworkInterface};
use pnet::packet::arp::{ArpHardwareTypes, ArpOperations, MutableArpPacket};
use pnet::packet::ethernet::{EtherTypes, MutableEthernetPacket};
use pnet::util::MacAddr;

use crate::config::constants::ARP_TIMEOUT_SECS;
use crate::schemas::arp::ArpResponse;

pub fn send_arp_request(
    target_ip: Ipv4Addr,
    interface_name: &str,
    timeout_secs: Option<f32>,
) -> Result<Option<ArpResponse>, Box<dyn std::error::Error + Send + Sync>> {
    let timeout_secs = timeout_secs.unwrap_or(ARP_TIMEOUT_SECS);
    let timeout_mili_secs = (timeout_secs * 1000.0) as u128;

    let interface_names_match = |iface: &NetworkInterface| iface.name == interface_name;

    // Find the network interface with the provided name
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .filter(interface_names_match)
        .next()
        .unwrap();

    // Get sender addresses (MAC and IP)
    let sender_mac = interface.mac.ok_or("Error getting MAC address")?;
    let sender_ip_canonical = interface
        .ips
        .iter()
        .find(|ip| ip.is_ipv4())
        .ok_or("Error getting IPv4 address")?
        .ip()
        .to_canonical();

    let sender_ip = match sender_ip_canonical {
        IpAddr::V4(ipv4) => ipv4,
        _ => return Err("Sender IP is not IPv4".into()),
    };

    let target_mac = MacAddr::broadcast();

    // 1. create arp buffer
    let mut arp_buffer = [0u8; 28];
    let mut arp_packet =
        MutableArpPacket::new(&mut arp_buffer).ok_or("Error creating the ARP Packet")?;

    arp_packet.set_hardware_type(ArpHardwareTypes::Ethernet);
    arp_packet.set_protocol_type(EtherTypes::Ipv4);
    arp_packet.set_hw_addr_len(6);
    arp_packet.set_proto_addr_len(4);
    arp_packet.set_operation(ArpOperations::Request);
    arp_packet.set_sender_hw_addr(sender_mac);
    arp_packet.set_sender_proto_addr(sender_ip);
    arp_packet.set_target_hw_addr(target_mac);
    arp_packet.set_target_proto_addr(target_ip);

    // 2. create ethernet frame
    let mut ethernet_buffer = [0u8; 14 + 42]; // 14 bytes for Ethernet header + 42 bytes for ARP packet 
    let mut ethernet_packet =
        MutableEthernetPacket::new(&mut ethernet_buffer).ok_or("Error creating Ethernet Packet")?;

    ethernet_packet.set_destination(target_mac);
    ethernet_packet.set_source(sender_mac);
    ethernet_packet.set_ethertype(EtherTypes::Arp);
    ethernet_packet.set_payload(&arp_buffer);

    // 3. send the packet
    let (mut tx, mut rx) = match datalink::channel(&interface, Config::default())? {
        Channel::Ethernet(tx, rx) => (tx, rx),
        _ => return Err("Error creating datalink channel".into()),
    };
    tx.send_to(&ethernet_buffer, None)
        .ok_or("Failed to send packet")??;

    let start_time = SystemTime::now();
    loop {
        if SystemTime::now().duration_since(start_time)?.as_millis() >= timeout_mili_secs {
            return Ok(None);
        }

        match rx.next() {
            Ok(packet) => {
                if let Some(arp_reply) = MutableArpPacket::new(&mut packet[14..].to_vec()) {
                    if arp_reply.get_operation() == ArpOperations::Reply
                        && arp_reply.get_sender_proto_addr() == target_ip
                    {
                        println!(
                            "Received ARP reply from IP: {:?}, MAC: {:?}",
                            arp_reply.get_sender_proto_addr(),
                            arp_reply.get_sender_hw_addr()
                        );
                        return Ok(Some(ArpResponse {
                            target_ip: arp_reply.get_sender_proto_addr().to_string(),
                            target_mac: arp_reply.get_sender_hw_addr().to_string(),
                            sender_ip: arp_reply.get_target_proto_addr().to_string(),
                            sender_mac: arp_reply.get_target_hw_addr().to_string(),
                            interface_name: interface_name.to_string(),
                            timeout_secs: Some(timeout_secs),
                        }));
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
            }
        }
    }
}
