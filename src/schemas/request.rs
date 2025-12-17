use std::collections::HashMap;
use std::convert::From;

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

pub enum Request {
    Http(HttpRequest),
    Https(HttpsRequest),
}

impl Request {
    pub fn get_headers(&self) -> HashMap<String, String> {
        match self {
            Request::Http(req) => req
                .headers
                .iter()
                .map(|(k, v)| {
                    (
                        k.as_str().to_string(),
                        v.to_str().unwrap_or_default().to_string(),
                    )
                })
                .collect(),
            Request::Https(req) => req.headers.clone(),
        }
    }
}

impl From<HttpRequest> for Request {
    fn from(req: HttpRequest) -> Self {
        Request::Http(req)
    }
}

impl From<HttpsRequest> for Request {
    fn from(req: HttpsRequest) -> Self {
        Request::Https(req)
    }
}
