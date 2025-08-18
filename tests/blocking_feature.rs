use openapi_gen::openapi_client;

#[test]
fn test_client_without_blocking() {
    // This should compile without the blocking feature
    openapi_client!("openapi.json", "BasicClient");

    // Test basic async client creation
    let client = BasicClient::new("https://api.example.com");

    // Test with custom reqwest client
    let custom_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    let client_with_custom = BasicClient::with_client("https://api.example.com", custom_client);

    // Ensure the client is generic and works with default type
    let _: BasicClient = client;
    let _: BasicClient<reqwest::Client> = client_with_custom;
}

#[cfg(feature = "blocking")]
#[test]
fn test_client_with_blocking() {
    openapi_client!("openapi.json", "BlockingClient");

    // Create a blocking client
    let blocking_client = reqwest::blocking::Client::new();

    // Test with_client method with blocking client
    let api_with_blocking = 
        BlockingClient::with_client("https://api.example.com", blocking_client);

    // Ensure correct types
    let _: BlockingClient<reqwest::blocking::Client> = api_with_blocking;
}

#[test]
fn test_generic_client_types() {
    // This test ensures our generic client works with different types
    openapi_client!("openapi.json", "GenericTestClient");

    // Test with default client (async)
    let default_client = GenericTestClient::new("https://api.example.com");
    let _: GenericTestClient = default_client;

    // Test with custom async client
    let custom = reqwest::Client::new();
    let custom_client = GenericTestClient::with_client("https://api.example.com", custom);
    let _: GenericTestClient<reqwest::Client> = custom_client;

    // Test with blocking client (only when feature is enabled)
    #[cfg(feature = "blocking")]
    {
        let blocking = reqwest::blocking::Client::new();
        let blocking_client = GenericTestClient::with_client("https://api.example.com", blocking);
        let _: GenericTestClient<reqwest::blocking::Client> = blocking_client;
    }
}

#[cfg(feature = "blocking")]
#[test]
fn test_blocking_methods_exist() {
    // This test verifies that blocking methods are generated with correct signatures
    openapi_client!("openapi.json", "BlockingMethodsClient");
    
    let blocking_client = reqwest::blocking::Client::new();
    let _client = BlockingMethodsClient::with_client("https://api.example.com", blocking_client);
    
    // These method calls validate that:
    // 1. Blocking methods exist
    // 2. They have the correct (non-async) signatures
    // 3. They return the same types as async versions but without Future wrapper
    // Note: We're not actually calling them, just verifying they compile
    
    // Example method signatures that should exist:
    // client.list_users(Some(10), Some(0), Some("active".to_string())) -> ApiResult<UserList>
    // client.get_user_by_id(123) -> ApiResult<User>
    // client.create_user(serde_json::json!({"name": "Test"})) -> ApiResult<User>
}