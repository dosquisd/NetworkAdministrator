use std::convert::Infallible;

use http_body_util::{BodyExt, Full};
use hyper::{Request, Response, body};
use uuid::Uuid;

pub async fn handler_example(
    _req: Request<body::Incoming>,
) -> Result<Response<Full<body::Bytes>>, Infallible> {
    println!("Received a request: {:?}", _req);
    Ok(Response::new(Full::new(body::Bytes::from("Hello, World!"))))
}

pub async fn handler_request(
    _req: Request<body::Incoming>,
) -> Result<Response<Full<body::Bytes>>, Infallible> {
    let req_id = Uuid::new_v4().to_string();

    // TODO: The request received from the client is a request from the browser to the proxy server.
    // we need to parse the request and forward it to the destination server.
    // and then we need to send the response back to the client.

    // First step. Parse the request and print its details.

    let method = _req.method().to_owned();
    let uri = _req.uri().to_owned();
    let version = _req.version().to_owned();
    let headers = _req.headers().to_owned();
    let body = _req.collect().await.unwrap().to_bytes();

    println!("Handling request: {}", req_id);
    println!("Request method: {:?}", method);
    println!("Request URI: {:?}", uri);
    println!("Request version: {:?}", version);
    println!("Request headers: {:?}", headers);
    println!("Request body: {:?}", body);
    println!();

    // Second step. Make the request to the destination server.

    // TODO: ...

    // Third step. Send the response back to the client.

    Ok(Response::new(Full::new(body::Bytes::from(
        "Request handled!",
    ))))
}
