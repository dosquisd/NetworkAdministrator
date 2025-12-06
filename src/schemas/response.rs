use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct HttpsResponse {
    pub version: String,
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}
