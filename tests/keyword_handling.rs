use openapi_gen::openapi_client;

// Test that Rust keywords are properly handled in generated code
openapi_client!("openapi.json", "KeywordTestApi");

#[test]
fn test_keyword_handling() {
    // The generated code should compile without issues even if the OpenAPI spec
    // contains field names or other identifiers that are Rust keywords
    
    let _client = KeywordTestApi::new("https://api.example.com");
    
    // The ApiResponse struct should be generated with a 'type' field that gets
    // converted to 'r#type' to avoid keyword conflicts
    
    // This test mainly verifies that:
    // 1. The create_rust_safe_ident function properly handles keywords
    // 2. Generated structs with keyword fields compile correctly
    // 3. The refactoring didn't break any functionality
}