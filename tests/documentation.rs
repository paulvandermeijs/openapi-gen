use openapi_gen::openapi_client;

// Generate client with documentation
openapi_client!("openapi.json", "DocumentedPetStoreApi");

#[test]
fn test_generated_documentation() {
    // Test that the client compiles and can be instantiated
    let _client = DocumentedPetStoreApi::new("https://api.example.com");
    
    // The generated structs and methods should have documentation
    // This test mainly verifies compilation with the new doc generation
}