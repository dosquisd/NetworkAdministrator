use crate::filters::is_domain_blacklisted;
use crate::schemas::Response;

pub fn analyze_and_modify_response(resp: &Response) -> Response {
    let mut modified_response = resp.clone();

    csp_stripping(&mut modified_response);

    if let Some(html) = resp.body_as_string() {
        let mut modified_html = remove_ad_scripts(&html);
        // modified_html = inject_mutation_observer(&modified_html);
        modified_html = inject_customs_script(
            &modified_html,
            "console.log('Injected script by Network Administrator');",
        );
        modified_response.set_body_str(&modified_html);
    }

    modified_response
}

fn csp_stripping(response: &mut Response) {
    let csp_headers = [
        "Content-Security-Policy",
        "X-Content-Security-Policy",
        "Content-Security-Policy-Report-Only",
        "X-WebKit-CSP",
    ];

    for header in csp_headers.iter() {
        response.headers_mut().remove(*header);
    }
}

pub fn remove_ad_scripts(html: &str) -> String {
    let internal_pattern = r#"(?s)<script(?![^>]*\ssrc\s*=)[^>]*>[^<]*\w+\s*=\s*window\.adsbygoogle[^<]*(?:<(?!/script>)[^<]*)*</script>"#;
    // `regex` crate does not support look-around, so we use `fancy_regex` here.
    let internal_re = fancy_regex::Regex::new(internal_pattern).unwrap();

    let external_pattern =
        r#"(?s)<script[^>]*\ssrc\s*=\s*["']?(?:https?:)?\/\/([^\/\s"']+)[^>]*>.*?<\/script>"#;
    let external_re = regex::Regex::new(external_pattern).unwrap();

    let mut modified_html = html.to_string();
    modified_html = internal_re.replace_all(&modified_html, "").to_string();

    for (script_snippet, [host]) in external_re.captures_iter(html).map(|c| c.extract()) {
        if is_domain_blacklisted(host) {
            modified_html = modified_html.replace(&script_snippet, "");
        }
    }

    modified_html.trim().to_string()
}

#[allow(dead_code, unused_variables)]
fn inject_mutation_observer(html: &str) -> String {
    // TODO: Implement ad script removal via MutationObserver
    // The following script is an example of how to use MutationObserver to remove ad scripts dynamically, but it's not functional
    // in this case, because the list of blacklisted domains is huge (+1M) and hardcoding them is not feasible.

    // I was thinking of creating a script to connect to a local endpoint to fetch the blacklist, but I'm not completely sure about that.
    // Another way could be to encode the blacklist in a compressed format and include it in the script (BloomFilter), but that
    // would bloat the script size significantly.

    // let script = r#"
    //     const observer = new MutationObserver((mutations) => {
    //         mutations.forEach((mutation) => {
    //             mutation.addedNodes.forEach((node) => {
    //                 if (node.tagName === 'SCRIPT' && node.src) {
    //                     const blacklistedDomains = ['adserver.com', 'tracker.net'];
    //                     blacklistedDomains.forEach((domain) => {
    //                         if (node.src.includes(domain)) {
    //                             console.log('Removing ad script:', node.src);
    //                             node.remove();
    //                         }
    //                     });
    //                 }
    //             });
    //         });
    //     });

    //     observer.observe(document.body, { childList: true, subtree: true });
    // "#;

    // inject_customs_script(html, script)
    todo!("Create a function that injects the mutation observer script into the HTML. Currently not implemented.");
}

pub fn inject_customs_script(html: &str, script: &str) -> String {
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
