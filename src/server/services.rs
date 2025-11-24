use std::convert::Infallible;

use http_body_util::{BodyExt, Full};
use hyper::{Request, Response, body};

#[warn(dead_code)]
pub async fn handler_connect(
    _req: Request<body::Incoming>,
) -> Result<Response<Full<body::Bytes>>, Infallible> {
    // This is a placeholder for handling CONNECT method
    // used for HTTPS tunneling.
    todo!("Implement CONNECT method handling");
}

#[tracing::instrument(level = "info", name = "HandlerRequest")]
pub async fn handler_request(
    req: Request<body::Incoming>,
) -> Result<Response<Full<body::Bytes>>, Infallible> {
    let req_id = uuid::Uuid::new_v4();

    tracing::info!("Received request ID {}", req_id);

    // First step. Parse the request and print its details.
    let method = req.method().to_owned();
    let uri = req.uri().to_owned();
    let version = req.version();
    let headers = req.headers().to_owned();
    let body = req.collect().await.unwrap().to_bytes();

    // Try to get the port from the URI to identify the scheme (http or https), this is because
    // the scheme is not always present in the URI.
    let scheme = match uri.scheme_str() {
        Some(scheme_str) => scheme_str,
        None => {
            let port = uri.port_u16();
            tracing::warn!("URI scheme not found, inferring from port -- {:?}", port);

            if port.unwrap_or(80) == 443 {
                "https"
            } else {
                "http"
            }
        }
    };

    // Second step. Make the request to the destination server.
    let client_builder = reqwest::ClientBuilder::new();
    let client_builder = match version {
        http::Version::HTTP_09 => client_builder.http09_responses(),
        http::Version::HTTP_2 => client_builder.http2_prior_knowledge(),
        // HTTP/1.0, HTTP/1.1, HTTP/3.0 (for the last one, I did not find the property methods)
        _ => client_builder.http1_only(),
    };

    let client = client_builder
        .build()
        .expect("Error creating the http client");

    // This should never failed if the server is acting as a proxy
    if let None = uri.authority() {
        tracing::error!("No authority found in the URI");
        return Ok(Response::new(Full::new(body::Bytes::from(
            "Error: No authority found in the URI",
        ))));
    }

    let url = format!(
        "{}://{}{}",
        scheme,
        uri.authority().unwrap(),
        uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/")
    );

    let reqwest_method = match method.as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        // "OPTIONS" => reqwest::Method::OPTIONS,
        "DELETE" => reqwest::Method::DELETE,
        "HEAD" => reqwest::Method::HEAD,
        // "CONNECT" => reqwest::Method::CONNECT,
        "PATCH" => reqwest::Method::PATCH,
        // "TRACE" => reqwest::Method::TRACE,
        _ => {
            tracing::warn!("Unsupported HTTP method: {}", method);
            reqwest::Method::GET
        }
    };

    tracing::info!(
        "Forwarding request ID {}: {} {}",
        req_id,
        reqwest_method,
        url
    );
    let response = client
        .request(reqwest_method, url)
        .headers(headers)
        .body(body)
        .send()
        .await;

    // Third step. Send the response back to the client.
    match response {
        Ok(resp) => {
            let status = resp.status();
            let resp_headers = resp.headers().clone();
            let resp_body = resp.bytes().await;

            match resp_body {
                Ok(bytes) => {
                    let mut builder = Response::builder().status(status);
                    for (key, value) in resp_headers.iter() {
                        builder = builder.header(key, value);
                    }

                    return Ok(builder.body(Full::new(body::Bytes::from(bytes))).unwrap());
                }
                Err(e) => {
                    let error_response = format!("Error reading response body: {}", e);
                    tracing::error!(error_response);
                    return Ok(Response::new(Full::new(body::Bytes::from(error_response))));
                }
            }
        }
        Err(err) => {
            tracing::error!("Error making request to destination server: {}", err);
            Ok(Response::new(Full::new(body::Bytes::from(format!(
                "Error: {}",
                err
            )))))
        }
    }
}
