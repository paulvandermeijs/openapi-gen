use openapi_gen::openapi_client;

#[test]
fn custom_client_name() {
    openapi_client!("openapi.json", "TestApi");

    let _api = TestApi::new("");
}

#[test]
fn auto_generated_client_name() {
    openapi_client!("openapi.json");

    // The API title is "OpenAPI Client Test API" so it should generate "OpenApiClientTestApiApi"
    let _api = OpenApiClientTestApiApi::new("");
}
