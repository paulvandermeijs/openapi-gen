use openapi_gen::openapi_client;

#[test]
fn test_param_structs_basic_functionality() {
    openapi_client!("openapi.json", use_param_structs = true);
    let _client = OpenApiClientTestApiApi::new("https://api.example.com");

    // Test struct creation and Default trait
    let _params = ListUsersParams::new();
    let _params: ListUsersParams = Default::default();

    // Test builder pattern
    let _params = ListUsersParams::new()
        .with_limit(10)
        .with_offset(0)
        .with_type("admin".to_string());

    // Test field assignment
    let mut params = ListUsersParams::new();
    params.limit = Some(5);
    params.offset = Some(10);
    params.r#type = Some("guest".to_string());
}

#[test]
fn test_param_structs_with_custom_client_name() {
    openapi_client!("openapi.json", "CustomClient", use_param_structs = true);
    let _client = CustomClient::new("https://api.example.com");

    let _params = ListUsersParams::new();
}

#[test]
fn test_required_parameters() {
    openapi_client!("openapi.json", use_param_structs = true);
    let _client = OpenApiClientTestApiApi::new("https://api.example.com");

    // Required path parameters
    let params = GetUserByIdParams::new(42i64);
    assert_eq!(params.user_id, 42i64);

    // Mixed required and optional parameters
    let params = GetPostCommentsParams::new("test-post".to_string());
    assert_eq!(params.post_id, "test-post");
    assert_eq!(params.self_, None);

    let params = GetPostCommentsParams::new("another-post".to_string()).with_self_(true);
    assert_eq!(params.post_id, "another-post");
    assert_eq!(params.self_, Some(true));

    // Field modification
    let mut params = GetPostCommentsParams::new("field-test".to_string());
    params.self_ = Some(false);
    params.post_id = "updated-post-id".to_string();
    assert_eq!(params.post_id, "updated-post-id");
}

#[test]
fn test_method_integration() {
    openapi_client!("openapi.json", use_param_structs = true);
    let client = OpenApiClientTestApiApi::new("https://api.example.com");

    // Test method calls with parameter structs
    let params = ListUsersParams::new().with_limit(10);
    let _result = client.list_users(params);

    let params = GetUserByIdParams::new(123i64);
    let _result = client.get_user_by_id(params);

    let params = GetPostCommentsParams::new("test-post".to_string()).with_self_(true);
    let _result = client.get_post_comments(params);

    // Test with request body
    let params = UpdateUserParams::new(456i64);
    let body = serde_json::json!({"username": "new_username"});
    let _result = client.update_user(params, body);
}

#[test]
fn test_all_parameter_types() {
    openapi_client!("openapi.json", use_param_structs = true);
    let client = OpenApiClientTestApiApi::new("https://api.example.com");

    // Only optional parameters
    let params = ListUsersParams::new();
    let _result = client.list_users(params);

    // Required i64 parameter
    let params = GetUserByIdParams::new(789i64);
    let _result = client.get_user_by_id(params);

    // Required String parameter
    let params = GetPostCommentsParams::new("test-post".to_string());
    let _result = client.get_post_comments(params);

    // Required parameter + request body
    let params = UpdateUserParams::new(999i64);
    let body = serde_json::json!({"email": "new@example.com"});
    let _result = client.update_user(params, body);

    // Required parameter for deletion
    let params = DeleteUserParams::new(111i64);
    let _result = client.delete_user(params);
}
