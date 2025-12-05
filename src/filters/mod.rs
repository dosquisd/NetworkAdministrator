// All operations related to filter domain management are handled in this module.
// including blacklisting for ads, and whitelisting domains to avoid TLS interception.

use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, LazyLock, RwLock},
};

use regex::Regex;
use serde::{Deserialize, Serialize};

static FILTER_PATH: &str = "./.proxy/filter.toml";

pub enum ListConfigType {
    Exact,
    Wildcard,
    Regex,
}

// TOML file content
#[derive(Debug, Serialize, Deserialize)]
struct FilterConfig {
    blacklist: ListConfig,
    whitelist: ListConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListConfig {
    #[serde(default)]
    exact: Vec<String>,
    #[serde(default)]
    wildcard: Vec<String>,
    #[serde(default)]
    regex: Vec<String>,
}

// Internal representation for efficient domain filtering
#[derive(Clone, Debug)]
struct DomainFilter {
    blacklist_exact: HashSet<String>,
    whitelist_exact: HashSet<String>,

    blacklist_wildcards: Vec<String>,
    whitelist_wildcards: Vec<String>,

    blacklist_regex: Vec<Regex>,
    whitelist_regex: Vec<Regex>,
}

impl DomainFilter {
    pub fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let path = PathBuf::from(FILTER_PATH);

        let content = std::fs::read_to_string(path)?;
        let config: FilterConfig = toml::from_str(&content)?;

        let blacklist_exact: HashSet<String> = config.blacklist.exact.into_iter().collect();
        let whitelist_exact: HashSet<String> = config.whitelist.exact.into_iter().collect();

        let blacklist_wildcards = config.blacklist.wildcard;
        let whitelist_wildcards = config.whitelist.wildcard;

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
            blacklist_exact,
            whitelist_exact,
            blacklist_wildcards,
            whitelist_wildcards,
            blacklist_regex,
            whitelist_regex,
        })
    }

    /// Add a domain to the specified list and dump the updated configuration to the TOML file.
    /// This operation is not meant to be used frequently, because the most common operation is read,
    /// then, this function is not optimized for perfomance right now, but it's a good idea to implement
    /// a more efficient way to handle frequent updates in the future.
    fn add_domain(&mut self, domain: &str, list_type: ListConfigType, is_blacklisted: bool) {
        match (list_type, is_blacklisted) {
            (ListConfigType::Exact, true) => {
                self.blacklist_exact.insert(domain.to_string());
            }
            (ListConfigType::Exact, false) => {
                self.whitelist_exact.insert(domain.to_string());
            }
            (ListConfigType::Wildcard, true) => {
                self.blacklist_wildcards.push(domain.to_string());
            }
            (ListConfigType::Wildcard, false) => {
                self.whitelist_wildcards.push(domain.to_string());
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

        // Dump to TOML file
        let filter_config = FilterConfig {
            blacklist: ListConfig {
                exact: self.blacklist_exact.iter().cloned().collect(),
                wildcard: self.blacklist_wildcards.clone(),
                regex: self
                    .blacklist_regex
                    .iter()
                    .map(|re| re.as_str().to_string())
                    .collect(),
            },
            whitelist: ListConfig {
                exact: self.whitelist_exact.iter().cloned().collect(),
                wildcard: self.whitelist_wildcards.clone(),
                regex: self
                    .whitelist_regex
                    .iter()
                    .map(|re| re.as_str().to_string())
                    .collect(),
            },
        };

        let toml_str = toml::to_string(&filter_config).unwrap();
        std::fs::write(FILTER_PATH, toml_str).unwrap();
    }

    pub fn add_domain_to_blacklist(&mut self, domain: &str, list_type: ListConfigType) {
        self.add_domain(domain, list_type, true);
    }

    pub fn add_domain_to_whitelist(&mut self, domain: &str, list_type: ListConfigType) {
        self.add_domain(domain, list_type, false);
    }

    fn is_listed(&self, domain: &str, is_blacklisted: bool) -> bool {
        if self.whitelist_exact.contains(domain) {
            return !is_blacklisted;
        }

        if self.blacklist_exact.contains(domain) {
            return is_blacklisted;
        }

        if self
            .whitelist_wildcards
            .iter()
            .any(|wc| domain.ends_with(wc.trim_start_matches('*')))
        {
            return !is_blacklisted;
        }

        if self
            .blacklist_wildcards
            .iter()
            .any(|wc| domain.ends_with(wc.trim_start_matches('*')))
        {
            return is_blacklisted;
        }

        if self.whitelist_regex.iter().any(|re| re.is_match(domain)) {
            return !is_blacklisted;
        }

        if self.blacklist_regex.iter().any(|re| re.is_match(domain)) {
            return is_blacklisted;
        }

        false
    }

    pub fn is_blacklisted(&self, domain: &str) -> bool {
        self.is_listed(domain, true)
    }

    pub fn is_whitelisted(&self, domain: &str) -> bool {
        self.is_listed(domain, false)
    }
}

static DOMAIN_FILTER: LazyLock<Arc<RwLock<DomainFilter>>> = LazyLock::new(|| {
    let filter = DomainFilter::load().unwrap_or_else(|e| {
        panic!("Failed to load domain filter configuration: {}", e);
    });
    Arc::new(RwLock::new(filter))
});

pub fn add_domain_to_blacklist(domain: &str, list_type: ListConfigType) {
    let mut filter = DOMAIN_FILTER.write().unwrap();
    filter.add_domain_to_blacklist(domain, list_type);
}

pub fn add_domain_to_whitelist(domain: &str, list_type: ListConfigType) {
    let mut filter = DOMAIN_FILTER.write().unwrap();
    filter.add_domain_to_whitelist(domain, list_type);
}

pub fn is_domain_blacklisted(domain: &str) -> bool {
    let filter = DOMAIN_FILTER.read().unwrap();
    filter.is_blacklisted(domain)
}

pub fn is_domain_whitelisted(domain: &str) -> bool {
    let filter = DOMAIN_FILTER.read().unwrap();
    filter.is_whitelisted(domain)
}

