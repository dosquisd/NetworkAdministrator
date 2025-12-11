import argparse
import os
import socket
from typing import Optional

from jinja2 import Environment, FileSystemLoader


def main(
    *,
    username: str,
    ip_address: str,
    out_dir: str,
    template_dir: Optional[str] = "templates/",
) -> None:
    environment = Environment(loader=FileSystemLoader(template_dir))
    template = environment.get_template("inventory.ini.j2")

    server_info = {
        "ip": ip_address,
        "user": username,
    }

    final_inventory = template.render(server=server_info)

    output_file = os.path.join(out_dir, "inventory.ini")
    with open(output_file, "w") as file:
        file.write(final_inventory)


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
        "--out-dir",
        type=str,
        default="ansible/",
        help="The directory where the generated inventory file will be saved.",
    )
    parser.add_argument(
        "--template-dir",
        type=str,
        default="templates/",
        help="The directory where the Jinja2 templates are located.",
    )

    args = parser.parse_args()
    main(
        username=args.username,
        ip_address=args.ip_address,
        out_dir=args.out_dir,
        template_dir=args.template_dir,
    )
