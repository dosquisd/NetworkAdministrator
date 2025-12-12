mod decoder;
mod types;

use std::time::SystemTime;

use pcap::{Capture, Device};

use crate::sniff::decoder::PacketDecoder;

pub fn sniff_network(
    interface: &str,
    duration: Option<u64>,
    count: Option<u32>,
    output_file: Option<&str>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!(
        "Sniffing on interface: {}, duration: {:?}, count: {:?}, output_file: {:?}",
        interface, duration, count, output_file
    );

    let (mut init, stop_condition): (u64, Box<dyn Fn(&mut u64) -> bool>) = {
        if duration.is_none() && count.is_none() {
            (0, Box::new(|_| true))
        } else if let Some(dur) = duration {
            let start = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_secs();
            (
                start,
                Box::new(move |init| {
                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    now - *init >= dur
                }),
            )
        } else {
            let cnt = count.unwrap() as u64;
            (
                0,
                Box::new(move |init| {
                    let return_value = *init >= cnt;
                    *init = *init + 1;
                    return_value
                }),
            )
        }
    };

    let device = Device::list()?
        .into_iter()
        .find(|d| d.name == interface)
        .ok_or_else(|| format!("Device {} not found", interface))?;

    let mut cap = Capture::from_device(device)?
        .promisc(true)
        .timeout(100)
        .open()?;

    let time_start = SystemTime::now();
    loop {
        let packet = cap.next_packet();
        match packet {
            Ok(pkt) => {
                let raw_bytes = pkt.data;
                let packet_decoder = PacketDecoder::from_packet(raw_bytes)?;

                println!("Packet data: {}", packet_decoder);
                println!("Decoded payload: {:?}", packet_decoder.decode_payload());
                println!("Raw bytes: {:02x?}", raw_bytes);
                println!("----------------------------------------");
            }
            Err(pcap::Error::NoMorePackets) => {
                println!("No more packets available.");
            }
            Err(e) => {
                eprintln!("Error capturing packet: {}", e);
            }
        }

        if stop_condition(&mut init) {
            break;
        }
    }

    println!(
        "Sniffing finished. Duration: {:?}",
        SystemTime::now().duration_since(time_start)?
    );
    Ok(())
}
