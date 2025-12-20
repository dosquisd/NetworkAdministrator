use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct HttpResponse {
    pub version: String,
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct HttpsResponse {
    pub version: String,
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

#[derive(Clone, Debug)]
pub enum Response {
    Http(HttpResponse),
    Https(HttpsResponse),
}

impl Response {
    pub fn headers(&self) -> HashMap<String, String> {
        match self {
            Response::Http(req) => req.headers.clone(),
            Response::Https(req) => req.headers.clone(),
        }
    }

    pub fn headers_mut(&mut self) -> &mut HashMap<String, String> {
        match self {
            Response::Http(req) => &mut req.headers,
            Response::Https(req) => &mut req.headers,
        }
    }

    pub fn body_as_string(&self) -> Option<String> {
        match self {
            Response::Http(req) => req
                .body
                .as_ref()
                .and_then(|b| String::from_utf8(b.clone()).ok()),
            Response::Https(req) => req
                .body
                .as_ref()
                .and_then(|b| String::from_utf8(b.clone()).ok()),
        }
    }

    pub fn set_body(&mut self, body: Vec<u8>) {
        match self {
            Response::Http(req) => req.body = Some(body),
            Response::Https(req) => req.body = Some(body),
        }
    }

    pub fn set_body_str(&mut self, body: &str) {
        self.set_body(body.as_bytes().to_vec());
    }
}

impl From<HttpResponse> for Response {
    fn from(req: HttpResponse) -> Self {
        Response::Http(req)
    }
}

impl From<HttpsResponse> for Response {
    fn from(req: HttpsResponse) -> Self {
        Response::Https(req)
    }
}

impl Into<HttpResponse> for Response {
    fn into(self) -> HttpResponse {
        match self {
            Response::Http(resp) => resp,

            // In this case, responses for HTTP and HTTPS are the same
            Response::Https(resp) => HttpResponse {
                version: resp.version,
                status_code: resp.status_code,
                status_text: resp.status_text,
                headers: resp.headers,
                body: resp.body,
            },
        }
    }
}

impl Into<HttpsResponse> for Response {
    fn into(self) -> HttpsResponse {
        match self {
            Response::Https(resp) => resp,

            // In this case, responses for HTTP and HTTPS are the same
            Response::Http(resp) => HttpsResponse {
                version: resp.version,
                status_code: resp.status_code,
                status_text: resp.status_text,
                headers: resp.headers,
                body: resp.body,
            },
        }
    }
}
