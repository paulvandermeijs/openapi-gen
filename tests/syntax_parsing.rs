use openapi_gen::openapi_client;

#[test]
fn test_syntax_parsing_only() {
    // Test that the parser accepts the new syntax without generating problematic code
    // By setting use_param_structs = false, we avoid the struct generation issues
    openapi_client!("openapi.json", use_param_structs = false);

    let _client = OpenApiClientTestApiApi::new("https://api.example.com");
}

#[test]
fn test_client_name_syntax() {
    // Test client name + option syntax
    openapi_client!("openapi.json", "TestClient", use_param_structs = false);

    let _client = TestClient::new("https://api.example.com");
}
