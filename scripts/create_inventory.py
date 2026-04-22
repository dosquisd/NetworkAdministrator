#!/usr/bin/env python
import argparse
import json
import secrets
import socket
import urllib.request
from pathlib import Path
from typing import List, Optional

from jinja2 import Environment, FileSystemLoader


url = "https://ipinfo.io"
try:
    with urllib.request.urlopen(url) as response:
        data = json.loads(response.read().decode())
        country_code = data.get("country")
except Exception as e:
    print(f"[!] Error fetching country code from {url}: {e}")
    country_code = "US"  # Default to US if the country code cannot be fetched

cwd = Path.cwd()
PROJECT_DIR = Path(__file__).parents[1].resolve()
DEFAULT_OUT_DIR = (PROJECT_DIR / "ansible").relative_to(cwd)
DEFAULT_TEMPLATE_DIR = (PROJECT_DIR / "templates").relative_to(cwd)
DEFAULT_WAN_INTERFACE = "eth0"
DEFAULT_LAN_INTERFACE = "wlan0"
DEFAULT_LAN_CIDR = "192.168.1.0/24"
DEFAULT_HOSTAPD_SSID = "Test SSID"
DEFAULT_SECONDARY_DNS_SERVER = "8.8.8.8"
DEFAULT_HOSTAPD_COUNTRY_CODE = country_code
# Generate a random passphrase for the Wi-Fi network
DEFAULT_HOSTAPD_WPA_PASSPHRASE = secrets.token_urlsafe(32)


def concatenate_binaries(array: List[int], fixed_length: int = 0) -> int:
    result = 0
    for byte in array:
        fixed_length = max(fixed_length, byte.bit_length())
        result = (result << fixed_length) | byte
    return result


def split_into_octets(value: int) -> List[int]:
    return list(map(lambda i: (value >> (24 - i * 8)) & 0xFF, range(4)))


def binary_to_ip(value: int) -> str:
    octets = split_into_octets(value)
    return ".".join(str(octet) for octet in octets)


def main(
    *,
    username: str,
    ip_address: str,
    out_dir: str,
    wan_interface: str,
    lan_interface: str,
    lan_cidr: str,
    lan_cidr_start: Optional[str] = None,
    lan_cidr_end: Optional[str] = None,
    lan_cidr_gateway: Optional[str] = None,
    lan_cidr_broadcast: Optional[str] = None,
    secondary_dns_server: str = DEFAULT_SECONDARY_DNS_SERVER,
    hostapd_ssid: str = DEFAULT_HOSTAPD_SSID,
    hostapd_country_code: str = DEFAULT_HOSTAPD_COUNTRY_CODE,
    hostapd_wpa_passphrase: str = DEFAULT_HOSTAPD_WPA_PASSPHRASE,
    template_dir: Optional[str] = None,
    verbose: bool = False,
) -> None:
    if template_dir is None:
        template_dir = (DEFAULT_TEMPLATE_DIR).resolve()  # type: ignore
    else:
        template_dir = Path(template_dir).resolve()  # type: ignore

    assert template_dir.is_dir(), (  # type: ignore
        f"Template directory '{template_dir}' does not exist or is not a directory."
    )

    if verbose:
        print(f"[+] Using template directory: {template_dir}")

    # Get parts of the LAN CIDR for later use in the DHCP configuration
    assert "/" in lan_cidr, (
        f"Invalid LAN CIDR format: '{lan_cidr}'. Expected format: 'IP_ADDRESS/SUBNET_MASK_BITS'"
    )

    ip, subnet_bits = lan_cidr.split("/")
    subnet_bits = int(subnet_bits)
    assert 0 < subnet_bits <= 32, (
        f"Invalid subnet mask bits: {subnet_bits}. Must be between 0 and 32."
    )
    assert ip.count(".") == 3, f"Invalid IP address format in LAN CIDR: '{ip}'"

    network_address = concatenate_binaries([int(octet) for octet in ip.split(".")], 8)

    # basically: /24 -> 255.255.255.0
    mask = 0xFFFFFFFF << (32 - subnet_bits)
    lan = binary_to_ip(network_address & mask)
    lan_cidr_netmask = binary_to_ip(mask)

    # When the gateway is not provided, then the first host in the LAN CIDR will be used as the default gateway
    if lan_cidr_gateway is None:
        first_host = network_address + 1
        lan_cidr_gateway = binary_to_ip(first_host)

    # When the gateway is not provided, then the last host in the LAN CIDR will be used as the default gateway
    if lan_cidr_broadcast is None:
        last_host = network_address | (0xFFFFFFFF >> subnet_bits)
        lan_cidr_broadcast = binary_to_ip(last_host)

    # If the start and end of the DHCP range are not provided, then the entire range of hosts in the LAN CIDR will be used,
    # excluding the network address, gateway, and broadcast address
    if lan_cidr_start is None:
        # +2 to skip the network address and the gateway
        lan_cidr_start = binary_to_ip(network_address + 2)

    if lan_cidr_end is None:
        lan_cidr_end = binary_to_ip(network_address | (0xFFFFFFFF >> subnet_bits) - 1)

    server_info = {
        "ip": ip_address,
        "user": username,
        "wan_interface": wan_interface,
        "lan_interface": lan_interface,
        "lan_cidr": f"{lan}/{subnet_bits}",
        "lan": lan,
        "lan_netmask": subnet_bits,
        "lan_cidr_netmask": lan_cidr_netmask,
        "lan_cidr_gateway": lan_cidr_gateway,
        "lan_cidr_broadcast": lan_cidr_broadcast,
        "lan_cidr_start": lan_cidr_start,
        "lan_cidr_end": lan_cidr_end,
        "hostapd_ssid": hostapd_ssid,
        "hostapd_country_code": hostapd_country_code,
        "hostapd_wpa_passphrase": hostapd_wpa_passphrase,
        "secondary_dns_server": secondary_dns_server,
    }

    if verbose:
        print(
            f"[+] Generating inventory content with the following server information: {server_info}"
        )

    output_file = (Path(out_dir) / "inventory.ini").resolve()
    environment = Environment(loader=FileSystemLoader(template_dir))  # type: ignore
    template = environment.get_template("inventory.ini.j2")
    final_inventory = template.render(server=server_info)
    if verbose:
        print(f"[+] Generated inventory content. Now writing to file {output_file}...")

    with open(output_file, "w") as file:
        file.write(final_inventory)

    if verbose:
        print(f"[+] Inventory file successfully written to {output_file}")


if __name__ == "__main__":
    hostname = socket.gethostname()
    default_ip = socket.gethostbyname(hostname)

    parser = argparse.ArgumentParser(
        description=("Generate an Ansible inventory file from a template using Jinja2.")
    )
    parser.add_argument("username", type=str, help="The username for the server.")
    parser.add_argument(
        "--dns-server",
        type=str,
        default=DEFAULT_SECONDARY_DNS_SERVER,
        help=f"The IP address of the secondary DNS server to use in the DHCP configuration (default: {DEFAULT_SECONDARY_DNS_SERVER}).",
    )
    parser.add_argument(
        "--hostapd-country-code",
        type=str,
        default=DEFAULT_HOSTAPD_COUNTRY_CODE,
        help=f"The country code for the hostapd configuration (default: {DEFAULT_HOSTAPD_COUNTRY_CODE}).",
    )
    parser.add_argument(
        "--hostapd-ssid",
        type=str,
        default=DEFAULT_HOSTAPD_SSID,
        help=f"The SSID for the hostapd configuration (default: {DEFAULT_HOSTAPD_SSID}).",
    )
    parser.add_argument(
        "--hostapd-wpa-passphrase",
        type=str,
        default=DEFAULT_HOSTAPD_WPA_PASSPHRASE,
        help=(
            "The WPA passphrase for the hostapd configuration (default: a randomly generated 32-character string). "
            "Make sure to use a strong passphrase to secure your Wi-Fi network."
        ),
    )
    parser.add_argument(
        "--ip-address",
        type=str,
        default=default_ip,
        help=f"The IP address of the server (default: your private ip is [{default_ip}]).",
    )
    parser.add_argument(
        "--lan-cidr",
        type=str,
        default=DEFAULT_LAN_CIDR,
        help=f"The CIDR notation for the LAN network (default: {DEFAULT_LAN_CIDR}).",
    )
    parser.add_argument(
        "--lan-cidr-gateway",
        type=str,
        default=None,
        help=(
            "The IP address of the gateway for the LAN. "
            "If not provided, it will default to the first host in the LAN CIDR."
        ),
    )
    parser.add_argument(
        "--lan-cidr-broadcast",
        type=str,
        default=None,
        help=(
            "The IP address of the broadcast address for the LAN. "
            "If not provided, it will default to the last host in the LAN CIDR."
        ),
    )
    parser.add_argument(
        "--lan-cidr-start",
        type=str,
        default=None,
        help=(
            "The starting IP address of the DHCP range for the LAN. "
            "If not provided, it will default to the first host in the LAN CIDR after the gateway."
        ),
    )
    parser.add_argument(
        "--lan-cidr-end",
        type=str,
        default=None,
        help=(
            "The ending IP address of the DHCP range for the LAN. "
            "If not provided, it will default to the last host in the LAN CIDR before the broadcast address."
        ),
    )
    parser.add_argument(
        "--lan-interface",
        type=str,
        default=DEFAULT_LAN_INTERFACE,
        help=f"The name of the LAN interface to configure the homebrew router (default: {DEFAULT_LAN_INTERFACE}).",
    )
    parser.add_argument(
        "--template-dir",
        type=str,
        default=DEFAULT_TEMPLATE_DIR,
        help=f"The directory where the Jinja2 templates are located (default: {DEFAULT_TEMPLATE_DIR}).",
    )
    parser.add_argument(
        "--out-dir",
        type=str,
        default=DEFAULT_OUT_DIR,
        help=f"The directory where the generated inventory file will be saved (default: {DEFAULT_OUT_DIR}).",
    )
    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="Enable verbose output for debugging purposes.",
    )
    parser.add_argument(
        "--wan-interface",
        type=str,
        default=DEFAULT_WAN_INTERFACE,
        help=f"The name of the WAN interface to configure the homebrew router (default: {DEFAULT_WAN_INTERFACE}).",
    )

    args = parser.parse_args()
    main(
        username=args.username,
        ip_address=args.ip_address,
        out_dir=args.out_dir,
        wan_interface=args.wan_interface,
        lan_interface=args.lan_interface,
        lan_cidr=args.lan_cidr,
        lan_cidr_gateway=args.lan_cidr_gateway,
        lan_cidr_broadcast=args.lan_cidr_broadcast,
        lan_cidr_start=args.lan_cidr_start,
        lan_cidr_end=args.lan_cidr_end,
        hostapd_ssid=args.hostapd_ssid,
        hostapd_country_code=args.hostapd_country_code,
        hostapd_wpa_passphrase=args.hostapd_wpa_passphrase,
        secondary_dns_server=args.dns_server,
        template_dir=args.template_dir,
        verbose=args.verbose,
    )
