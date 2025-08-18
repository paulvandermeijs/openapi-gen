use super::{OpenApiInput, fetch_url_content, is_url, is_yaml_format};
use openapiv3::OpenAPI;

/// Load and parse an OpenAPI specification from file or URL
pub fn load_openapi_spec(input: &OpenApiInput) -> Result<OpenAPI, String> {
    // Read and parse the OpenAPI spec from file or URL
    let spec_content = if is_url(&input.spec_path) {
        fetch_url_content(&input.spec_path)?
    } else {
        std::fs::read_to_string(&input.spec_path)
            .map_err(|e| format!("Failed to read spec file: {}", e))?
    };

    let spec: OpenAPI = if is_yaml_format(&input.spec_path) {
        serde_yaml::from_str(&spec_content).map_err(|e| format!("Failed to parse YAML: {}", e))?
    } else {
        serde_json::from_str(&spec_content).map_err(|e| format!("Failed to parse JSON: {}", e))?
    };

    Ok(spec)
}
