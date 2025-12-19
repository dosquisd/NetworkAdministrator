use crate::schemas::HttpsResponse;

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
