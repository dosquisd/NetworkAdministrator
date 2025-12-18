// All operations related to filter domain management are handled in this module.
// including blacklisting for ads, and whitelisting domains to avoid TLS interception.

use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex, RwLock},
};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::config::constants::CONFIG_PATH;

static FILTER_PATH: LazyLock<PathBuf> = LazyLock::new(|| CONFIG_PATH.join("filter.toml"));
static UPDATE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum ListConfigType {
    Exact,
    Wildcard,
    Regex,
}

// TOML file content
#[derive(Debug, Default, Serialize, Deserialize)]
struct FilterConfig {
    blacklist: ListConfig,
    whitelist: ListConfig,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ListConfig {
    #[serde(default)]
    exact: Vec<String>,

    #[serde(default)]
    wildcard: Vec<String>,

    #[serde(default)]
    regex: Vec<String>,
}

// Internal representation for efficient domain filtering
#[derive(Clone, Debug, Default)]
struct DomainFilter {
    pub file: PathBuf,

    pub blacklist_exact: HashSet<String>,
    pub whitelist_exact: HashSet<String>,

    pub blacklist_wildcards: HashSet<String>,
    pub whitelist_wildcards: HashSet<String>,

    pub blacklist_regex: Vec<Regex>,
    pub whitelist_regex: Vec<Regex>,
}

impl DomainFilter {
    pub fn load(file: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let file = file.unwrap_or(FILTER_PATH.clone());
        let content = std::fs::read_to_string(&file).unwrap_or_default();
        let config: FilterConfig = toml::from_str(&content).unwrap_or_default();

        let blacklist_exact: HashSet<String> = config.blacklist.exact.into_iter().collect();
        let whitelist_exact: HashSet<String> = config.whitelist.exact.into_iter().collect();

        let blacklist_wildcards: HashSet<String> = config.blacklist.wildcard.into_iter().collect();
        let whitelist_wildcards: HashSet<String> = config.whitelist.wildcard.into_iter().collect();

        let blacklist_regex: Vec<Regex> = config
            .blacklist
            .regex
            .into_iter()
            .filter_map(|r| Regex::new(&r).ok())
            .collect();
        let whitelist_regex: Vec<Regex> = config
            .whitelist
            .regex
            .into_iter()
            .filter_map(|r| Regex::new(&r).ok())
            .collect();

        Ok(DomainFilter {
            file,
            blacklist_exact,
            whitelist_exact,
            blacklist_wildcards,
            whitelist_wildcards,
            blacklist_regex,
            whitelist_regex,
        })
    }

    /// Dump the current configuration to the TOML file, acquiring the update lock.
    fn dump_file(
        &self,
        save_backup: Option<bool>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _update_guard = UPDATE_LOCK.lock().unwrap();
        self.dump_file_unsafe(save_backup.unwrap_or(false))
    }

    /// Dump the current configuration to the TOML file without acquiring the update lock.
    fn dump_file_unsafe(
        &self,
        save_backup: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let filter_config = FilterConfig {
            blacklist: ListConfig {
                exact: self.blacklist_exact.iter().cloned().collect(),
                wildcard: self.blacklist_wildcards.iter().cloned().collect(),
                regex: self
                    .blacklist_regex
                    .iter()
                    .map(|re| re.as_str().to_string())
                    .collect(),
            },
            whitelist: ListConfig {
                exact: self.whitelist_exact.iter().cloned().collect(),
                wildcard: self.whitelist_wildcards.iter().cloned().collect(),
                regex: self
                    .whitelist_regex
                    .iter()
                    .map(|re| re.as_str().to_string())
                    .collect(),
            },
        };

        let toml_str = toml::to_string(&filter_config)?;
        if let Some(parent) = self.file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Save backup before overwriting if needed
        if save_backup {
            let backup_path = self.file.with_extension("toml.backup");
            std::fs::copy(&self.file, &backup_path)?;
        }

        // Atomic write
        let temp_path = self.file.with_extension("tmp");
        std::fs::write(&temp_path, toml_str)?;
        std::fs::rename(&temp_path, &self.file)?; // rename is atomic on POSIX systems

        Ok(())
    }

    /// Merge another DomainFilter into this one internally.
    fn merge_internal(&mut self, other: &DomainFilter) {
        self.blacklist_exact
            .extend(other.blacklist_exact.iter().cloned());
        self.whitelist_exact
            .extend(other.whitelist_exact.iter().cloned());

        self.blacklist_wildcards
            .extend(other.blacklist_wildcards.iter().cloned());
        self.whitelist_wildcards
            .extend(other.whitelist_wildcards.iter().cloned());

        self.blacklist_regex
            .extend(other.blacklist_regex.iter().cloned());
        self.whitelist_regex
            .extend(other.whitelist_regex.iter().cloned());
    }

    /// Replace the current filter with another one internally.
    fn replace_internal(&mut self, other: &Self) {
        *self = other.clone();
    }

    /// Merge another DomainFilter into this one and dump the updated configuration to the TOML file.
    pub fn merge(
        &mut self,
        other: &DomainFilter,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _update_guard = UPDATE_LOCK.lock().unwrap();
        self.merge_internal(other);
        self.dump_file_unsafe(true)
    }

    /// Replace the current filter with another one and persist to disk.
    pub fn replace(
        &mut self,
        other: &Self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _update_guard = UPDATE_LOCK.lock().unwrap();
        self.replace_internal(other);
        self.dump_file_unsafe(true)
    }

    /// Add a domain to the specified list and dump the updated configuration to the TOML file.
    /// This operation is not meant to be used frequently, because the most common operation is read,
    /// then, this function is not optimized for perfomance right now, but it's a good idea to implement
    /// a more efficient way to handle frequent updates in the future.
    pub fn add_domain(
        &mut self,
        domain: &str,
        list_type: ListConfigType,
        is_blacklisted: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match (list_type, is_blacklisted) {
            (ListConfigType::Exact, true) => {
                self.blacklist_exact.insert(domain.to_string());
            }
            (ListConfigType::Exact, false) => {
                self.whitelist_exact.insert(domain.to_string());
            }
            (ListConfigType::Wildcard, true) => {
                self.blacklist_wildcards.insert(domain.to_string());
            }
            (ListConfigType::Wildcard, false) => {
                self.whitelist_wildcards.insert(domain.to_string());
            }
            (ListConfigType::Regex, true) => {
                if let Ok(re) = Regex::new(domain) {
                    self.blacklist_regex.push(re);
                }
            }
            (ListConfigType::Regex, false) => {
                if let Ok(re) = Regex::new(domain) {
                    self.whitelist_regex.push(re);
                }
            }
        }

        self.dump_file(None)
    }

    pub fn remove_domain(
        &mut self,
        domain: &str,
        list_type: ListConfigType,
        is_blacklisted: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match (list_type, is_blacklisted) {
            (ListConfigType::Exact, true) => {
                self.blacklist_exact.remove(domain);
            }
            (ListConfigType::Exact, false) => {
                self.whitelist_exact.remove(domain);
            }
            (ListConfigType::Wildcard, true) => {
                self.blacklist_wildcards.retain(|d| d != domain);
            }
            (ListConfigType::Wildcard, false) => {
                self.whitelist_wildcards.retain(|d| d != domain);
            }
            (ListConfigType::Regex, true) => {
                self.blacklist_regex.retain(|re| re.as_str() != domain);
            }
            (ListConfigType::Regex, false) => {
                self.whitelist_regex.retain(|re| re.as_str() != domain);
            }
        }

        // Dump to TOML file
        self.dump_file(None)
    }

    pub fn is_listed(&self, domain: &str, is_blacklisted: bool) -> bool {
        match is_blacklisted {
            true => {
                if self.blacklist_exact.contains(domain) {
                    return true;
                }

                if self
                    .blacklist_wildcards
                    .iter()
                    .any(|wc| domain.ends_with(wc.trim_start_matches('*')))
                {
                    return true;
                }

                if self.blacklist_regex.iter().any(|re| re.is_match(domain)) {
                    return true;
                }
            }
            false => {
                if self.whitelist_exact.contains(domain) {
                    return true;
                }

                if self
                    .whitelist_wildcards
                    .iter()
                    .any(|wc| domain.ends_with(wc.trim_start_matches('*')))
                {
                    return true;
                }

                if self.whitelist_regex.iter().any(|re| re.is_match(domain)) {
                    return true;
                }
            }
        }

        false
    }
}

static DOMAIN_FILTER: LazyLock<Arc<RwLock<DomainFilter>>> = LazyLock::new(|| {
    let filter = DomainFilter::load(None).unwrap_or_else(|e| {
        panic!("Failed to load domain filter configuration: {}", e);
    });
    Arc::new(RwLock::new(filter))
});

pub fn add_domain_to_blacklist(
    domain: &str,
    list_type: ListConfigType,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut filter = DOMAIN_FILTER.write().unwrap();
    filter.add_domain(domain, list_type, true)
}

pub fn add_domain_to_whitelist(
    domain: &str,
    list_type: ListConfigType,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut filter = DOMAIN_FILTER.write().unwrap();
    filter.add_domain(domain, list_type, false)
}

pub fn is_domain_blacklisted(domain: &str) -> bool {
    let filter = DOMAIN_FILTER.read().unwrap();
    filter.is_listed(domain, true)
}

pub fn is_domain_whitelisted(domain: &str) -> bool {
    let filter = DOMAIN_FILTER.read().unwrap();
    filter.is_listed(domain, false)
}

pub fn remove_domain_from_blacklist(
    domain: &str,
    list_type: ListConfigType,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut filter = DOMAIN_FILTER.write().unwrap();
    filter.remove_domain(domain, list_type, true)
}

pub fn remove_domain_from_whitelist(
    domain: &str,
    list_type: ListConfigType,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut filter = DOMAIN_FILTER.write().unwrap();
    filter.remove_domain(domain, list_type, false)
}

pub fn get_blacklist(config_type: ListConfigType) -> Vec<String> {
    let filter = DOMAIN_FILTER.read().unwrap();
    match config_type {
        ListConfigType::Exact => filter.blacklist_exact.iter().cloned().collect(),
        ListConfigType::Wildcard => filter.blacklist_wildcards.iter().cloned().collect(),
        ListConfigType::Regex => filter
            .blacklist_regex
            .iter()
            .map(|re| re.as_str().to_string())
            .collect(),
    }
}

pub fn get_whitelist(config_type: ListConfigType) -> Vec<String> {
    let filter = DOMAIN_FILTER.read().unwrap();
    match config_type {
        ListConfigType::Exact => filter.whitelist_exact.iter().cloned().collect(),
        ListConfigType::Wildcard => filter.whitelist_wildcards.iter().cloned().collect(),
        ListConfigType::Regex => filter
            .whitelist_regex
            .iter()
            .map(|re| re.as_str().to_string())
            .collect(),
    }
}

/// Merge entries from an external file into the current filter.
pub fn merge_from_file(file: PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load and validate the file first (fails fast if invalid)
    let other_filter = DomainFilter::load(Some(file))?;

    // Apply changes with write lock
    let mut filter = DOMAIN_FILTER.write().unwrap();
    filter.merge(&other_filter)
}

/// Replace the entire filter from an external file.
pub fn replace_from_file(file: PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load and validate the file first
    let new_filter = DomainFilter::load(Some(file))?;

    // Replace with write lock
    let mut filter = DOMAIN_FILTER.write().unwrap();
    filter.replace(&new_filter)
}
