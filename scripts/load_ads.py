import argparse
import itertools
import tomllib
from pathlib import Path
from typing import Dict, List, Set

import tomli_w

PROJECT_PATH = Path(__file__).parents[1]
CONFIG_PATH = PROJECT_PATH / ".config"
ADS_LIST_PATH = PROJECT_PATH / "ads-list"


def load_current_config() -> Dict[str, str]:
    with open(CONFIG_PATH / "filter.toml", "rb") as f:
        config = tomllib.load(f)

    return config


def save_current_config(config: Dict[str, str]) -> None:
    # Different files to avoid conflicts with running filter used by the proxy
    save_path = CONFIG_PATH / "filter.updated.toml"
    print(f"Saving updated config to {save_path}")
    with open(save_path, "wb") as f:
        tomli_w.dump(config, f)


def process_line(line: str) -> str:
    # There are lines that start with IP addresses followed by the domain without spaces (fuck)
    ips = {"127.0.0.1", "0.0.0.0", "::1"}
    for ip in ips:
        if not line.startswith(f"{ip}"):
            continue
        line = line.removeprefix(f"{ip}").strip()

    # Some ads have commentaries next to the host, e.g.,
    if "#" in line:
        line = line.split("#", maxsplit=1)[0].strip()

    return line.strip()


def process_file(file: Path) -> Set[str]:
    with open(file, "r", encoding="utf-8") as f:
        ads = f.readlines()
        ads = {
            process_line(ad)
            for ad in ads
            if ad.strip() and not ad.startswith("#") and "localhost" not in ad
        }
    return ads


def load_ads_list(ads_list: Path) -> List[str]:
    def _add_host(file: Path) -> None:
        nonlocal ads_host
        ads = process_file(file)
        ads_host.update(ads)

    files = itertools.chain(ads_list.rglob("*.txt"), ads_list.rglob("hosts"))
    ads_host = set()
    list(map(lambda file: _add_host(file), files))

    return sorted(ads_host)


def main(ads_list: Path, force_update: bool = False) -> None:
    current_config = load_current_config()
    blacklist_exact = current_config["blacklist"]["exact"]  # type: ignore
    print(f"Current blacklist exact entries: {len(blacklist_exact):,}")

    loaded_ads = load_ads_list(ads_list)
    print(f"Loaded {len(loaded_ads):,} ads")

    if force_update:
        print("Force update enabled, replacing current config with loaded ads...")
        current_config["blacklist"]["exact"] = sorted(loaded_ads)  # type: ignore
    else:
        print("\nMerging ads into current config...")
        current_config["blacklist"]["exact"] = sorted(  # type: ignore
            set(loaded_ads) | set(blacklist_exact)
        )

    diff_size = len(current_config["blacklist"]["exact"]) - len(blacklist_exact)  # type: ignore
    print(f"Diff blacklist exact entries: {diff_size:,}")

    save_current_config(current_config)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Load ads from the list and update the config.")
    parser.add_argument(
        "ads-list",
        type=Path,
        default=ADS_LIST_PATH,
        help="Path to the ads list directory (default: %(default)s)",
    )
    parser.add_argument(
        "-f",
        "--force-update",
        action="store_true",
        help="Force update the config with loaded ads, replacing existing entries.",
    )
    return parser.parse_args()


if __name__ == "__main__":
    args = parse_args()
    main(ads_list=args.ads_list, force_update=args.force_update)
