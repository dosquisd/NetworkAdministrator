use std::fs;

use certgenutil::generate_server_cert_by_ca_pem;
use openssl::{asn1::Asn1Time, pkey::PKey, x509::X509};
use rcgen::{CertificateParams, KeyPair};
use time::{Duration, OffsetDateTime};

use crate::config::{
    CERT_DAYS_VALID, CERT_PATH, ProxyConfig, get_global_config, set_global_config,
};

pub fn generate_ca() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Generating new CA certificate at {:?}", CERT_PATH);

    let mut params = CertificateParams::new(vec!["localhost".to_string()])?;

    // Configure lifetime
    params.not_before = OffsetDateTime::now_utc();
    params.not_after = OffsetDateTime::now_utc() + Duration::days(CERT_DAYS_VALID as i64);

    // Configure as CA
    params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

    // Generate key pair and certificate
    // as a reminder: key_pair is a private key, and cert is the public certificate
    let key_pair = KeyPair::generate()?;
    let cert = params.self_signed(&key_pair)?;

    // Save to disk
    fs::create_dir_all(CERT_PATH.as_os_str())?;
    fs::write(CERT_PATH.join("ca_key.pem"), key_pair.serialize_pem())?;
    fs::write(CERT_PATH.join("ca_cert.pem"), cert.pem())?;

    Ok(cert.pem())
}

pub fn get_ca_cert() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let ca_cert_path = CERT_PATH.join("ca_cert.pem");
    let ca_cert = fs::read_to_string(ca_cert_path)?;

    // Make sure the CA is still valid
    let res = X509::from_pem(ca_cert.as_bytes())?;

    if res.not_after() < Asn1Time::days_from_now(30)? {
        tracing::warn!(
            "CA certificate expires soon! Expiration date: {:?}",
            res.not_after()
        );
        tracing::warn!("Please regenerate CA and redistribute to clients");
    }

    if res.not_after() < Asn1Time::days_from_now(0)? {
        tracing::error!("CA certificate has expired");
        // ca_cert = Arc::new(RwLock::new(generate_ca()?));
        let global_config = get_global_config();
        let updated_config = ProxyConfig {
            intercept_tls: false,
            block_ads: global_config.block_ads,
            cache_enabled: global_config.cache_enabled,
        };

        set_global_config(updated_config);
        return Err("CA certificate has expired, please regenerate it")?;
    }

    Ok(ca_cert)
}

pub fn get_ca_key() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let ca_key_path = CERT_PATH.join("ca_key.pem");
    let ca_key = fs::read_to_string(ca_key_path)?;
    Ok(ca_key)
}

pub fn generate_cert_for_domain(
    domain: &str,
) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
    fn from_vec_to_string(vec: Vec<u8>) -> String {
        String::from_utf8(vec).expect("Error converting vec to string")
    }

    let ca_cert = get_ca_cert()?;
    let ca_key = get_ca_key()?;

    let ca = format!("{}{}", ca_cert, ca_key);
    let (cert_pem, key_pem) = generate_server_cert_by_ca_pem(
        ca.as_str(),
        domain,
        1,
        vec!["localhost".to_string(), domain.to_string()],
    )?;

    // Parse cert to re-encode it in PEM format
    let cert_der = cert_pem.first().expect("Error loading the resulting cert");
    let cert = X509::from_der(cert_der)?;
    let cert_pem = from_vec_to_string(cert.to_pem()?);

    // Parse key to re-encode it in PEM format
    let pkey = PKey::private_key_from_der(key_pem.secret_der())?;
    let key_pem = from_vec_to_string(pkey.private_key_to_pem_pkcs8()?);

    Ok((cert_pem, key_pem))
}
