use std::path::PathBuf;

use super::domain_filter::{DOMAIN_FILTER, DomainFilter, ListConfigType};

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
