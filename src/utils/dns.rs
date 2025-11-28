use std::sync::LazyLock;

use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};

pub static DNS_RESOLVER: LazyLock<TokioAsyncResolver> =
    LazyLock::new(|| TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default()));
