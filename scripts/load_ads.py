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
    with open(CONFIG_PATH / "filter.updated.toml", "wb") as f:
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


def load_ads_list() -> List[str]:
    files = ADS_LIST_PATH.rglob("*.txt")
    ads_host = set()
    for file in files:
        ads = process_file(file)
        ads_host.update(ads)

    return sorted(ads_host)


def main() -> None:
    current_config = load_current_config()
    blacklist_exact = current_config["blacklist"]["exact"]
    print(f"Current blacklist exact entries: {len(blacklist_exact):,}")

    loaded_ads = load_ads_list()
    print(f"Loaded {len(loaded_ads):,} ads")

    print("\nMerging ads into current config...")
    current_config["blacklist"]["exact"] = sorted(
        set(loaded_ads) | set(blacklist_exact)
    )

    diff_size = len(current_config["blacklist"]["exact"]) - len(blacklist_exact)
    print(f"Diff blacklist exact entries: {diff_size:,}")

    save_current_config(current_config)


if __name__ == "__main__":
    main()
