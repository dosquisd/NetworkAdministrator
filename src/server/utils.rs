use crate::config::{ProxyConfig, get_global_config};
use crate::filters::is_domain_whitelisted;

pub fn intercept_https_request(host: &str, config: Option<ProxyConfig>) -> bool {
    let config = config.unwrap_or_else(get_global_config);

    if !config.intercept_tls {
        return false;
    }

    let is_host_whitelisted = is_domain_whitelisted(host);
    if is_host_whitelisted {
        tracing::info!("The host {} is whitelisted, not intercepting", host);
        return false;
    }

    // TODO: Implement here more complex logic to decide whether to intercept or not
    // using volatile databases like memcached, ttls, sliding windows, with these possible states:
    // 1. TRUSTED (it's possible to intercept)  ->  TTl: 5-10 minutes
    // 2. UNTRUSTED (it's not possible to intercept)  ->  TTL: 30 minutes
    // 3. UNKNOWN (the first time we see this host, we can decide to intercept or not,
    // and then store the result in the database for future requests)
    // 4. PROBING (temporary state to avoid race conditions when we are probing a host)  ->  TTL: 3-6 seconds

    // Then, how we probe a host? I think that the best way is to make a little handshake with the client,
    // and then decide whether to intercept or not, without the client even knows that we are doing this.
    // That means, if the client does not trust our CA, then the handshake will fail, we mark it as UNTRUSTED
    // we don't intercept (we just make a tunnel), and all without the client have to restart the connection.

    // Meanwhile, we intercept the traffic always, but this will be changed
    true
}