use std::fs;

use network_administrator::{config::CERT_PATH, utils::tls};

fn restore_old_file(
    old_filename: &str,
    new_filename: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let old_filepath = CERT_PATH.join(old_filename);
    let new_filepath = CERT_PATH.join(new_filename);

    fs::remove_file(new_filepath.clone())?;

    if old_filepath.exists() {
        fs::rename(old_filepath, new_filepath)?;
    }
    Ok(())
}

fn verify_integrity(new_content: &str, old_content_filename: &str) -> bool {
    let tmp_filepath = CERT_PATH.join(old_content_filename);
    if tmp_filepath.exists() {
        let old_content = fs::read_to_string(tmp_filepath).unwrap();
        return new_content != old_content;
    }
    true
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Check if CA already exists and rename them if necessary
    let ca_cert_filename = "ca_cert.pem";
    let ca_key_filename = "ca_key.pem";
    let ca_cert_old_filename = "old_ca_cert.pem";
    let ca_key_old_filename = "old_ca_key.pem";

    // This fails if the files don't exist, which is fine
    fs::rename(
        CERT_PATH.join(ca_cert_filename),
        CERT_PATH.join(ca_cert_old_filename),
    )
    .ok();
    fs::rename(
        CERT_PATH.join(ca_key_filename),
        CERT_PATH.join(ca_key_old_filename),
    )
    .ok();

    // Generate new CA
    let (new_ca_cert, new_ca_key) = tls::generate_ca()?;

    // Verify that the files were created and are different from the old ones
    match (
        verify_integrity(&new_ca_cert, ca_cert_old_filename),
        verify_integrity(&new_ca_key, ca_key_old_filename),
    ) {
        (true, true) => Ok(()),
        _ => {
            restore_old_file(ca_cert_old_filename, ca_cert_old_filename)?;
            restore_old_file(ca_key_old_filename, ca_key_filename)?;
            Err("Generated CA is identical to the old one, aborting to prevent overwriting.".into())
        }
    }
}
