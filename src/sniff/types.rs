#[derive(Debug, Clone)]
pub enum EtherType {
    Ipv4,
    Arp,
    #[allow(dead_code)]
    Unknown(u16),
}

impl EtherType {
    pub fn from_u16(value: u16) -> Self {
        match value {
            0x0800 => EtherType::Ipv4,
            0x0806 => EtherType::Arp,
            // 0x86DD => EtherType::Ipv6,
            other => EtherType::Unknown(other),
        }
    }

    pub fn to_u16(&self) -> u16 {
        match self {
            EtherType::Ipv4 => 0x0800,
            EtherType::Arp => 0x0806,
            // EtherType::Ipv6 => 0x86DD,
            EtherType::Unknown(value) => *value,
        }
    }
}
