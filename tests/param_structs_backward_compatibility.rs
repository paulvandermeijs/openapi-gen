use openapi_gen::openapi_client;

#[test]
fn test_individual_parameters_still_work() {
    // Ensure backward compatibility when use_param_structs = false
    openapi_client!("openapi.json", use_param_structs = false);
    let client = OpenApiClientTestApiApi::new("https://api.example.com");

    // Test that individual parameters still work
    let _result = client.list_users(Some(10), Some(0), Some("admin"));
    let _result = client.get_user_by_id(123i64);
    let _result = client.get_post_comments("test-post", Some(true));

    let body = serde_json::json!({"username": "test"});
    let _result = client.update_user(456i64, body);
    let _result = client.delete_user(789i64);
}

#[test]
fn test_default_syntax_compatibility() {
    openapi_client!("openapi.json");
    let _client = OpenApiClientTestApiApi::new("https://api.example.com");
}

#[test]
fn test_client_name_syntax_compatibility() {
    openapi_client!("openapi.json", "CustomClient");
    let _client = CustomClient::new("https://api.example.com");
}

#[test]
fn test_explicit_false_syntax_compatibility() {
    openapi_client!("openapi.json", use_param_structs = false);
    let _client = OpenApiClientTestApiApi::new("https://api.example.com");
}

#[test]
fn test_client_name_with_false_syntax_compatibility() {
    openapi_client!("openapi.json", "CustomClient", use_param_structs = false);
    let _client = CustomClient::new("https://api.example.com");
}
