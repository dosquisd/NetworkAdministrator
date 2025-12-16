use std::{collections::HashMap, path::PathBuf, sync::LazyLock};

use indicatif::{ProgressBar, ProgressStyle};

use crate::config::constants::CONFIG_PATH;

pub static KNOWN_MACS_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| CONFIG_PATH.join("known_macs.json"));

pub fn load_known_macs() -> HashMap<String, String> {
    let path = KNOWN_MACS_PATH.clone();
    if !path.exists() {
        return HashMap::new();
    }

    let content = std::fs::read_to_string(path.clone()).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn configure_progress_bar(length: u64) -> ProgressBar {
    let pb = ProgressBar::new(length);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    pb
}
