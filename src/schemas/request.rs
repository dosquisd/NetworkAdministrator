use http::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::str::FromStr;

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

#[derive(Clone, Debug)]
pub enum Request {
    Http(HttpRequest),
    Https(HttpsRequest),
}

impl Request {
    pub fn headers(&self) -> HashMap<String, String> {
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

    pub fn uri(&self) -> String {
        match self {
            Request::Http(req) => req.uri.to_string(),
            Request::Https(req) => req.uri.clone(),
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

impl Into<HttpRequest> for Request {
    fn into(self) -> HttpRequest {
        match self {
            Request::Http(req) => req,
            Request::Https(req) => {
                let version = match req.version.as_str() {
                    "HTTP/0.9" => http::Version::HTTP_09,
                    "HTTP/1.0" => http::Version::HTTP_10,
                    "HTTP/1.1" => http::Version::HTTP_11,
                    "HTTP/2.0" => http::Version::HTTP_2,
                    "HTTP/3.0" => http::Version::HTTP_3,
                    _ => http::Version::HTTP_11,
                };

                let headers = HeaderMap::try_from(&req.headers).ok().unwrap_or_else(|| {
                    let mut map = HeaderMap::new();
                    for (k, v) in req.headers {
                        let header_name = HeaderName::from_str(&k)
                            .expect(format!("Error parsing header name ({})", k).as_str());
                        let header_value = HeaderValue::from_str(&v)
                            .expect(format!("Error parsing header value ({})", v).as_str());
                        map.insert(header_name, header_value);
                    }

                    map
                });

                HttpRequest {
                    method: req.method,
                    uri: req.uri.parse().unwrap_or_default(),
                    version,
                    headers,
                    body: req.body.map(|b| bytes::Bytes::from(b)),
                }
            }
        }
    }
}

impl Into<HttpsRequest> for Request {
    fn into(self) -> HttpsRequest {
        match self {
            Request::Https(req) => req,
            Request::Http(req) => HttpsRequest {
                method: req.method,
                version: format!("{:?}", req.version),
                uri: req.uri.to_string(),
                headers: req
                    .headers
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.as_str().to_string(),
                            v.to_str().unwrap_or_default().to_string(),
                        )
                    })
                    .collect(),
                body: req.body.map(|b| String::from_utf8_lossy(&b).to_string()),
            },
        }
    }
}
