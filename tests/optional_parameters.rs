use openapi_gen::openapi_client;

#[test]
fn test_optional_parameters_compilation() {
    // This test validates that optional parameters work correctly by checking compilation
    openapi_client!("openapi.json", "ParameterTestApi");

    let client = ParameterTestApi::new("https://api.example.com");

    // Test path parameter (required)
    let _user_result = client.get_user_by_id(123);

    // Test optional query parameters - all should be Option<T>
    let _users_result_1 = client.list_users(None, None, None);
    let _users_result_2 = client.list_users(Some(10), None, None);
    let _users_result_3 = client.list_users(Some(10), Some(0), Some("admin"));

    // Test mixed optional parameters
    let _comments_result_1 = client.get_post_comments("post123", None);
    let _comments_result_2 = client.get_post_comments("post123", Some(true));
}

#[test]
fn test_parameter_types() {
    // This test ensures the parameter types are correct
    openapi_client!("openapi.json", "TypeTestApi");

    let client = TypeTestApi::new("https://api.example.com");

    // Check that string parameters accept &str (not String)
    let _user_result = client.get_user_by_id(123); // i64 parameter
    let _comments_result = client.get_post_comments("post123", Some(true)); // &str and Option<bool>

    // Check that optional parameters can be None
    let _users_result = client.list_users(None, None, None);

    // Check that optional string parameters accept &str
    let _users_with_type = client.list_users(Some(10), Some(0), Some("admin"));
}

/// This test would fail to compile if parameters were not properly optional
#[test]
fn test_optional_query_parameters_are_truly_optional() {
    openapi_client!("openapi.json", "OptionalTestApi");

    let client = OptionalTestApi::new("https://api.example.com");

    // According to OpenAPI spec, query parameters without "required: true" should be optional
    // Our openapi.json has these query parameters in /users endpoint:
    // - limit: Optional (no required field)
    // - offset: Optional (no required field)
    // - type: Optional (no required field)

    // This should compile because all query parameters are optional
    let _result = client.list_users(None, None, None);

    // Path parameters are always required, so this takes i64 directly (not Option<i64>)
    let _user = client.get_user_by_id(123);

    // Mixed scenario: path param required, query param optional
    let _comments = client.get_post_comments("post123", None);
}
