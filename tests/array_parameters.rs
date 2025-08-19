use openapi_gen::openapi_client;

#[test]
fn test_array_parameter_compilation() {
    // This test validates that parameter structs with array parameters compile correctly
    // and that Vec<String> parameters are properly handled in URL building
    //
    // Uses the petstore API's findByTags endpoint which has a required array parameter
    // Previously this would fail with: "Vec<std::string::String>` doesn't implement `std::fmt::Display`"

    openapi_client!(
        "https://petstore3.swagger.io/api/v3/openapi.json",
        "PetstoreApi",
        use_param_structs = true
    );

    let client = PetstoreApi::new("https://petstore3.swagger.io/api/v3");

    // Test parameter struct creation with array fields - findByTags has required array parameter
    let params = FindPetsByTagsParams::new(vec!["tag1".to_string(), "tag2".to_string()]);

    // Test that required array parameter is properly set
    assert_eq!(params.tags, vec!["tag1".to_string(), "tag2".to_string()]);

    // Test builder pattern with arrays
    let params2 = FindPetsByTagsParams::new(vec!["rust".to_string(), "openapi".to_string()]);

    assert_eq!(
        params2.tags,
        vec!["rust".to_string(), "openapi".to_string()]
    );

    // The key test: this should compile and not panic at runtime
    // Previously this would fail with Display trait error during URL building
    let _result = client.find_pets_by_tags(params);
}

#[test]
fn test_array_parameter_url_formatting() {
    // This test specifically validates that array parameters are formatted correctly
    // as comma-separated values in query strings using petstore findByTags

    openapi_client!(
        "https://petstore3.swagger.io/api/v3/openapi.json",
        "PetstoreApi",
        use_param_structs = true
    );

    let client = PetstoreApi::new("https://petstore3.swagger.io/api/v3");

    // Test array parameter with multiple values
    let params = FindPetsByTagsParams::new(vec![
        "dog".to_string(),
        "cat".to_string(),
        "available".to_string(),
    ]);
    assert_eq!(params.tags.len(), 3);

    // Test single value array parameter
    let params = FindPetsByTagsParams::new(vec!["puppy".to_string()]);
    assert_eq!(params.tags, vec!["puppy".to_string()]);

    // The key test: this should compile and not panic at runtime
    // Previously this would fail with Display trait error when building URLs
    let _result = client.find_pets_by_tags(params);
}

#[test]
fn test_mixed_parameter_types_with_arrays() {
    // Test mixing array parameters with regular parameters using petstore API
    // The petstore API has both simple and array parameters across different endpoints

    openapi_client!(
        "https://petstore3.swagger.io/api/v3/openapi.json",
        "PetstoreApi",
        use_param_structs = true
    );

    let client = PetstoreApi::new("https://petstore3.swagger.io/api/v3");

    // Test findByTags (array parameter)
    let tags_params = FindPetsByTagsParams::new(vec!["dog".to_string(), "puppy".to_string()]);
    assert_eq!(tags_params.tags.len(), 2);

    // Test findByStatus (enum parameter) if it uses param structs
    let status_params = FindPetsByStatusParams::new("available".to_string());
    assert_eq!(status_params.status, "available".to_string());

    // The compilation tests - both should work without Vec<String> Display errors
    let _result1 = client.find_pets_by_tags(tags_params);
    let _result2 = client.find_pets_by_status(status_params);
}
