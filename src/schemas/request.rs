#[derive(Debug, Clone)]
pub struct HTTPRequestSchema {
    pub method: String,
    pub uri: http::Uri,
    pub version: http::Version,
    pub headers: http::HeaderMap,
    pub body: Option<bytes::Bytes>,
}

impl HTTPRequestSchema {
    pub fn new(
        method: String,
        uri: http::Uri,
        version: http::Version,
        headers: http::HeaderMap,
        body: Option<bytes::Bytes>,
    ) -> Self {
        Self {
            method,
            uri,
            version,
            headers,
            body,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HTTPSRequestSchema {
    pub method: String,
    pub uri: String,
    pub version: String,
    pub headers: Vec<String>,
    pub body: Option<bytes::Bytes>,
}

impl HTTPSRequestSchema {
    pub fn new(
        method: String,
        uri: String,
        version: String,
        headers: Vec<String>,
        body: Option<bytes::Bytes>,
    ) -> Self {
        Self {
            method,
            uri,
            version,
            headers,
            body,
        }
    }
}
