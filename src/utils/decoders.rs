use std::io::Read;

use flate2::read::{DeflateDecoder, GzDecoder};

pub fn decode_brotli(
    compressed: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut decompressed = Vec::new();
    brotli::BrotliDecompress(&mut &compressed[..], &mut decompressed)?;
    Ok(decompressed)
}

pub fn decode_gzip(compressed: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

pub fn decode_deflate(
    compressed: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut decoder = DeflateDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    Ok(decompressed)
}

pub fn decode_zstd(compressed: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let decompressed = zstd::decode_all(&compressed[..])?;
    Ok(decompressed)
}
