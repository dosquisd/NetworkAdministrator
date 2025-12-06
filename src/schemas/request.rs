use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub uri: http::Uri,
    pub version: http::Version,
    pub headers: http::HeaderMap,
    pub body: Option<bytes::Bytes>,
}


#[derive(Clone, Debug)]
pub struct HttpsRequest {
    pub method: String,
    pub version: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}
