// TODO: Implement parallel scanning for improved performance.

mod arp;
use arp::send_arp_request;

use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::LazyLock;

use crate::cli::types::OutputFormat;
use crate::config::constants::CONFIG_PATH;

static KNOWN_MACS_PATH: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_PATH.join("known_macs.json"));

fn load_known_macs() -> HashMap<String, String> {
    let path = KNOWN_MACS_PATH.clone();
    if !path.exists() {
        return HashMap::new();
    }

    let content = std::fs::read_to_string(path.clone()).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

/// Scans a given IPv4 address and prints the result.
/// The IP address should be in the following format: "xxx.xxx.xxx.xxx/x".
pub fn scan_network(
    network_address_v4: &str,
    interface_name: &str,
    timeout_secs: Option<f32>,
    output_format: OutputFormat,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let timeout_secs = match timeout_secs {
        Some(timeout) => {
            if timeout > 0.0 {
                Some(timeout)
            } else {
                None
            }
        }
        None => None,
    };

    let parts: Vec<&str> = network_address_v4.split('/').collect();
    if parts.len() != 2 {
        return Err("Invalid IP address format. Expected format: xxx.xxx.xxx.xxx/x".into());
    }

    let ip = parts[0];
    let subnet_mask: u8 = parts[1].parse()?;
    if subnet_mask > 32 || subnet_mask < 8 {
        return Err("Invalid subnet mask. It should be between 8 and 32.".into());
    }

    let octets: Vec<u8> = ip.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    if octets.len() != 4 {
        return Err("Invalid IP address format.".into());
    }

    let mask = !0u32 << (32 - subnet_mask);
    let network_address = u32::from_be_bytes([octets[0], octets[1], octets[2], octets[3]]) & mask;

    let broadcast_address = network_address | !mask;
    let first_host = network_address + 1;
    let last_host = broadcast_address - 1;
    let first_octet = octets[0];

    let first_second_octet = (first_host >> 16) & 0xFF;
    let first_third_octet = (first_host >> 8) & 0xFF;
    let first_fourth_octet = first_host & 0xFF;

    let last_second_octet = (last_host >> 16) & 0xFF;
    let last_third_octet = (last_host >> 8) & 0xFF;
    let last_fourth_octet = last_host & 0xFF;

    println!(
        "Scanning from {}.{}.{}.{} to {}.{}.{}.{} -> Total Hosts: {}\n",
        first_octet,
        first_second_octet,
        first_third_octet,
        first_fourth_octet,
        first_octet,
        last_second_octet,
        last_third_octet,
        last_fourth_octet,
        last_host - first_host + 1
    );

    let all_combinations = (first_second_octet..=last_second_octet)
        .flat_map(|second| {
            (first_third_octet..=last_third_octet).flat_map(move |third| {
                (first_fourth_octet..=last_fourth_octet)
                    .map(move |fourth| format!("{}.{}.{}.{}", first_octet, second, third, fourth))
            })
        })
        .collect::<Vec<String>>();

    if verbose {
        println!(
            "Loading known MAC addresses from {:?}",
            KNOWN_MACS_PATH.clone()
        );
    }
    let known_macs = load_known_macs();
    if verbose {
        println!("Loaded {} known MAC addresses.\n", known_macs.len());
    }

    let mut arp_responses = Vec::new();
    for target_ip in all_combinations {
        if verbose {
            print!("ARP request to {} ", target_ip);
            io::stdout().flush()?;
        }

        let arp_response = send_arp_request(target_ip.parse()?, interface_name, timeout_secs)?;
        if let Some(mut response) = arp_response {
            if verbose {
                println!("[{}]", response.target_mac);
            }

            let known_name = known_macs.get(&response.target_mac);
            if let Some(name) = known_name {
                response.alias = Some(name.clone());
            }

            arp_responses.push(response);
        } else {
            if verbose {
                println!("[host down]");
            }
        }
    }

    if arp_responses.is_empty() {
        println!("No ARP responses received.");
        return Ok(());
    }

    if verbose {
        println!(
            "\nDisplaying results in {} format:",
            output_format.to_string()
        );
    }

    output_format.show_scanning_results(&arp_responses);

    Ok(())
}
