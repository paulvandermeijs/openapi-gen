use openapi_gen::openapi_client;

// Test with PartialEq derive
openapi_client!(
    "openapi.json",
    "TestApi",
    struct_attrs = (derive(PartialEq))
);

#[test]
fn test_struct_with_partial_eq() {
    // Test that generated structs have PartialEq
    let user1 = User {
        id: 1,
        username: "test".to_string(),
        email: "test@example.com".to_string(),
        status: UserStatus::Active,
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        age: Some(25),
        height: None,
        weight: None,
        is_active: Some(true),
        r#type: Some("user".to_string()),
        tags: Some(vec!["developer".to_string()]),
        metadata: None,
        profile: None,
        preferences: None,
        created_at: Some("2024-01-01".to_string()),
        last_login: None,
    };

    let user2 = User {
        id: 1,
        username: "test".to_string(),
        email: "test@example.com".to_string(),
        status: UserStatus::Active,
        first_name: Some("Test".to_string()),
        last_name: Some("User".to_string()),
        age: Some(25),
        height: None,
        weight: None,
        is_active: Some(true),
        r#type: Some("user".to_string()),
        tags: Some(vec!["developer".to_string()]),
        metadata: None,
        profile: None,
        preferences: None,
        created_at: Some("2024-01-01".to_string()),
        last_login: None,
    };

    // This should compile because PartialEq is derived
    assert_eq!(user1, user2);
}

#[test]
fn test_enum_with_partial_eq() {
    let status1 = UserStatus::Active;
    let status2 = UserStatus::Active;

    // This should compile because PartialEq is derived
    assert_eq!(status1, status2);
}
