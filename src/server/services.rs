use http_body_util::Full;
use hyper::body::{self, Bytes};
use hyper::{Request, Response};

use std::convert::Infallible;

pub async fn handler_example(_req: Request<body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    println!("Received a request: {:?}", _req);
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))

}
