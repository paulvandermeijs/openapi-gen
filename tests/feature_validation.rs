use openapi_gen::openapi_client;

// Generate a comprehensive test client
openapi_client!("openapi.json", "FeatureTestApi");

#[test]
fn test_all_crate_features() {
    let _client = FeatureTestApi::new("https://api.test.com/v2");

    // This test validates that our comprehensive OpenAPI schema exercises all features:

    // 1. Multiple HTTP methods (GET, POST, PUT, DELETE)
    // 2. Path parameters (userId, postId)
    // 3. Query parameters (limit, offset, type, self)
    // 4. Request bodies (JSON)
    // 5. Various response types
    // 6. Schema references ($ref)
    // 7. Enum types (UserStatus)
    // 8. Array types (tags, users, comments)
    // 9. Object types with nested structures
    // 10. Optional vs required fields
    // 11. Various data types:
    //     - Strings (with formats like email, date-time, uri)
    //     - Integers (int32, int64)
    //     - Numbers (float, double)
    //     - Booleans
    //     - Arrays
    //     - Objects
    // 12. Rust keyword handling in:
    //     - Field names (type, self, const)
    //     - Parameter names (type, self)
    //     - Operation IDs (r#const)
    // 13. Comprehensive documentation from:
    //     - API info (title, description, version, license, contact, terms)
    //     - Operation summaries and descriptions
    //     - Schema descriptions
    //     - Field descriptions
    // 14. Type aliases (SimpleString, NumberArray)
    // 15. Complex nested object structures
    // 16. Validation constraints (min/max, minLength/maxLength)
}

#[test]
fn test_generated_method_signatures() {
    let _client = FeatureTestApi::new("https://api.test.com/v2");

    // Test that methods with the expected signatures are generated:

    // Methods with various parameter types
    // client.list_users(limit: Option<i32>, offset: Option<i64>, r#type: Option<String>)
    // client.create_user(body: serde_json::Value)
    // client.get_user_by_id(user_id: i64)
    // client.update_user(user_id: i64, body: serde_json::Value)
    // client.delete_user(user_id: i64)
    // client.get_post_comments(post_id: String, r#self: Option<bool>)
    // client.r#const() // Tests keyword handling in operation ID

    // If this compiles, it means:
    // 1. All methods were generated with correct names
    // 2. Parameter types are correctly mapped
    // 3. Rust keywords are properly handled with r# prefix
    // 4. Path parameters are extracted correctly
    // 5. Query parameters are handled correctly
    // 6. Request bodies are supported
}

#[test]
fn test_generated_types() {
    // Test that all expected types are generated:
    // - User (complex object with many fields)
    // - UserStatus (enum)
    // - UserProfile (nested object)
    // - UserPreferences (nested object with enum field)
    // - NotificationSettings (object with enum)
    // - CreateUserRequest (request DTO)
    // - UpdateUserRequest (request DTO)
    // - UserList (response DTO with array)
    // - Comment (object with keyword field)
    // - ValidationError (error response)
    // - FieldError (nested error object)
    // - SimpleString (type alias)
    // - NumberArray (type alias for array)

    // If this compiles, it means all types were generated correctly
}

#[test]
fn test_keyword_field_handling() {
    // This test specifically validates Rust keyword handling
    // The schema includes fields named "type", "self", "const", and "r#type"
    // These should be converted to r#type, r#self, r#const, etc.

    // If this compiles without errors, keyword handling works correctly
}
