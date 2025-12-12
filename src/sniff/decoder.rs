use std::{collections::HashMap, fmt};

use super::types::EtherType;

#[derive(Debug, Clone)]
pub struct PacketDecoder {
    pub dst_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ethertype: EtherType,
    pub payload: Vec<u8>,
}

impl PacketDecoder {
    pub fn from_packet(packet: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Here you would implement the actual parsing logic for Layer 2 packets.
        // For demonstration purposes, we'll just print the length of the data.
        let dst_mac = &packet[0..6];
        let src_mac = &packet[6..12];
        let ethertype = u16::from_be_bytes([packet[12], packet[13]]);
        let payload = packet[14..].to_vec();

        Ok(PacketDecoder {
            dst_mac: dst_mac.try_into().unwrap(),
            src_mac: src_mac.try_into().unwrap(),
            ethertype: EtherType::from_u16(ethertype),
            payload,
        })
    }

    pub fn decode_payload(&self) -> HashMap<String, Vec<u8>> {
        match self.ethertype {
            EtherType::Ipv4 => self.decode_payload_ipv4(),
            EtherType::Arp => self.decode_payload_arp(),
            EtherType::Unknown(_) => Default::default(),
        }
    }

    fn format_mac(mac: &[u8; 6]) -> String {
        mac.iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<String>>()
            .join(":")
    }

    pub fn dst_mac_to_string(&self) -> String {
        PacketDecoder::format_mac(&self.dst_mac)
    }

    pub fn src_mac_to_string(&self) -> String {
        PacketDecoder::format_mac(&self.src_mac)
    }

    fn decode_payload_ipv4(&self) -> HashMap<String, Vec<u8>> {
        let mut decoded_payload = HashMap::new();

        decoded_payload.insert("IpVersion".to_string(), [self.payload[0]].to_vec());
        decoded_payload.insert("Ttl".to_string(), [self.payload[8]].to_vec());
        decoded_payload.insert("Protocol".to_string(), [self.payload[9]].to_vec());
        decoded_payload.insert("SrcIp".to_string(), self.payload[12..16].to_vec());
        decoded_payload.insert("DstIp".to_string(), self.payload[16..20].to_vec());

        decoded_payload
    }

    fn decode_payload_arp(&self) -> HashMap<String, Vec<u8>> {
        let mut decoded_payload = HashMap::new();

        decoded_payload.insert("HardwareType".to_string(), self.payload[0..2].to_vec());
        decoded_payload.insert("ProtocolType".to_string(), self.payload[2..4].to_vec());
        decoded_payload.insert("HardwareSize".to_string(), [self.payload[4]].to_vec());
        decoded_payload.insert("ProtocolSize".to_string(), [self.payload[5]].to_vec());
        decoded_payload.insert("Opcode".to_string(), self.payload[6..8].to_vec());
        decoded_payload.insert("SenderMac".to_string(), self.payload[8..14].to_vec());
        decoded_payload.insert("SenderIp".to_string(), self.payload[14..18].to_vec());
        decoded_payload.insert("TargetMac".to_string(), self.payload[18..24].to_vec());
        decoded_payload.insert("TargetIp".to_string(), self.payload[24..28].to_vec());

        decoded_payload
    }
}

impl fmt::Display for PacketDecoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Dst MAC: {}, Src MAC: {}, EtherType: {:?}, Payload Length: {} bytes",
            self.dst_mac_to_string(),
            self.src_mac_to_string(),
            self.ethertype,
            self.payload.len()
        )
    }
}
