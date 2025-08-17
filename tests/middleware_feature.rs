use openapi_gen::openapi_client;

#[test]
fn test_client_without_middleware() {
    // This should compile without the middleware feature
    openapi_client!("openapi.json", "BasicClient");

    // Test basic client creation
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

#[cfg(feature = "middleware")]
#[test]
fn test_client_with_middleware() {
    use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

    openapi_client!("openapi.json", "MiddlewareClient");

    // Create a middleware client
    let middleware_client = ClientBuilder::new(reqwest::Client::new()).build();

    // Test with_client method with middleware
    let api_with_client =
        MiddlewareClient::with_client("https://api.example.com", middleware_client);

    // Ensure correct types
    let _: MiddlewareClient<ClientWithMiddleware> = api_with_client;
}

#[test]
fn test_generic_client_types() {
    // This test ensures our generic client works with different types
    openapi_client!("openapi.json", "GenericTestClient");

    // Test with default client
    let default_client = GenericTestClient::new("https://api.example.com");
    let _: GenericTestClient = default_client;

    // Test with custom client
    let custom = reqwest::Client::new();
    let custom_client = GenericTestClient::with_client("https://api.example.com", custom);
    let _: GenericTestClient<reqwest::Client> = custom_client;
}
