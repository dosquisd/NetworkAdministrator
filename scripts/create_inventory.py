#!/usr/bin/env python
import argparse
import socket
from pathlib import Path
from typing import Optional

from jinja2 import Environment, FileSystemLoader


cwd = Path.cwd()
PROJECT_DIR = Path(__file__).parents[1].resolve()
DEFAULT_OUT_DIR = (PROJECT_DIR / "ansible").relative_to(cwd)
DEFAULT_TEMPLATE_DIR = (PROJECT_DIR / "templates").relative_to(cwd)
DEFAULT_WAN_INTERFACE = "eth0"
DEFAULT_LAN_INTERFACE = "wlan0"
DEFAULT_LAN_CIDR = "192.168.1.0/24"


def main(
    *,
    username: str,
    ip_address: str,
    out_dir: str,
    wan_interface: str,
    lan_interface: str,
    lan_cidr: str,
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

    output_file = (Path(out_dir) / "inventory.ini").resolve()
    environment = Environment(loader=FileSystemLoader(template_dir))  # type: ignore
    template = environment.get_template("inventory.ini.j2")

    server_info = {
        "ip": ip_address,
        "user": username,
        "wan_interface": wan_interface,
        "lan_interface": lan_interface,
        "lan_cidr": lan_cidr,
    }

    if verbose:
        print(f"[+] Generating inventory content with the following server information: {server_info}")

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
        template_dir=args.template_dir,
        verbose=args.verbose,
    )
