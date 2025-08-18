/// Check if a path is a URL (starts with http:// or https://)
pub fn is_url(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://")
}

/// Check if a path indicates YAML format (file extension or URL path)
pub fn is_yaml_format(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    path_lower.ends_with(".yaml") || path_lower.ends_with(".yml")
}

/// Fetch content from a URL at compile time
pub fn fetch_url_content(url: &str) -> Result<String, String> {
    // Use blocking reqwest for compile-time execution
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;

    rt.block_on(async {
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch URL {}: {}", url, e))?;

        if !response.status().is_success() {
            return Err(format!(
                "HTTP error {} when fetching {}",
                response.status(),
                url
            ));
        }

        response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))
    })
}
