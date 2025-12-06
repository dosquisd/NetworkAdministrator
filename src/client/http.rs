use std::convert::Infallible;

use bytes::Bytes;
use http::Response;
use http_body_util::Full;
use uuid::Uuid;

use crate::schemas::HttpRequest;

#[tracing::instrument(level = "info", name = "ForwardHTTPRequest", skip(req_params))]
pub async fn forward_http_request(
    req_id: Uuid,
    req_params: HttpRequest,
) -> Result<Response<Full<Bytes>>, Infallible> {
    static SCHEME: &str = "http";

    let client_builder = reqwest::ClientBuilder::new();
    let client_builder = match req_params.version {
        http::Version::HTTP_09 => client_builder.http09_responses(),
        http::Version::HTTP_2 => client_builder.http2_prior_knowledge(),
        // HTTP/1.0, HTTP/1.1, HTTP/3.0 (for the last one, I did not find the property methods)
        _ => client_builder.http1_only(),
    };

    let client = client_builder
        .build()
        .expect("Error creating the http client");

    // This should never failed if the server is acting as a proxy
    if req_params.uri.authority().is_none() {
        tracing::error!("No authority found in the URI");
        return Ok(Response::new(Full::new(Bytes::from(
            "Error: No authority found in the URI",
        ))));
    }

    let url = format!(
        "{}://{}{}",
        SCHEME,
        req_params.uri.authority().unwrap(),
        req_params
            .uri
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/")
    );

    let reqwest_method = match req_params.method.as_str() {
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
            tracing::warn!("Unsupported HTTP method: {}", req_params.method);
            reqwest::Method::GET
        }
    };

    tracing::info!(
        "Forwarding request ID {}: {} {}",
        req_id,
        reqwest_method,
        url
    );

    let request_builder = client
        .request(reqwest_method, url)
        .headers(req_params.headers);
    let request_builder = match req_params.body {
        None => request_builder,
        Some(body) => request_builder.body(body),
    };

    let response = request_builder.send().await;

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

                    return Ok(builder.body(Full::new(bytes)).unwrap());
                }
                Err(e) => {
                    let error_response = format!("Error reading response body: {e}");
                    tracing::error!(error_response);
                    return Ok(Response::new(Full::new(Bytes::from(error_response))));
                }
            }
        }
        Err(err) => {
            tracing::error!("Error making request to destination server: {}", err);
            Ok(Response::new(Full::new(Bytes::from(format!(
                "Error: {err}"
            )))))
        }
    }
}
