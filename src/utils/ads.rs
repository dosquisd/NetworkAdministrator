use crate::filters::is_domain_blacklisted;
use crate::schemas::{HttpsRequest, HttpsResponse};

pub fn analyze_and_modify_request(req: &HttpsRequest) -> HttpsRequest {
    // TODO: Implement ad-blocking logic here.
    req.clone()
}

/// Checks if the given HTTP request is an ad request based on its URI.
pub fn is_ad_request_based_on_uri(uri: &str) -> bool {
    let host = uri.split(':').next().unwrap_or_default();
    is_domain_blacklisted(host)
}

pub fn analyze_and_modify_response(resp: &HttpsResponse) -> HttpsResponse {
    // TODO: Implement ad content removal logic here.
    resp.clone()
}

pub fn inject_script(html: &str, script: &str) -> String {
    let injection = format!("<script>{}</script>", script);
    if let Some(pos) = html.rfind("</body>") {
        let mut modified_html = String::with_capacity(html.len() + injection.len());
        modified_html.push_str(&html[..pos]);
        modified_html.push_str(&injection);
        modified_html.push_str(&html[pos..]);
        modified_html
    } else {
        let mut modified_html = String::with_capacity(html.len() + injection.len());
        modified_html.push_str(html);
        modified_html.push_str(&injection);
        modified_html
    }
}
