use openapi_gen::openapi_client;

#[test]
fn petstore_from_url() {
    // Load Petstore OpenAPI spec from official Swagger URL
    openapi_client!(
        "https://petstore3.swagger.io/api/v3/openapi.json",
        "PetstoreApi"
    );

    // Verify the client can be instantiated
    let _api = PetstoreApi::new("https://petstore3.swagger.io/api/v3");
}

#[tokio::test]
async fn petstore_fetch_single_pet() {
    // Load Petstore OpenAPI spec from official Swagger URL
    openapi_client!(
        "https://petstore3.swagger.io/api/v3/openapi.json",
        "PetstoreApi"
    );

    // Create the client
    let client = PetstoreApi::new("https://petstore3.swagger.io/api/v3");

    // Try to fetch a pet by ID - should succeed for ID 1
    match client.get_pet_by_id(1).await {
        Ok(pet) => {
            println!("Successfully fetched pet: {:?}", pet);
        }
        Err(e) => {
            panic!("Failed to fetch pet by ID 1: {:?}", e);
        }
    }
}

#[tokio::test]
async fn petstore_fetch_available_pets() {
    // Load Petstore OpenAPI spec from official Swagger URL
    openapi_client!(
        "https://petstore3.swagger.io/api/v3/openapi.json",
        "PetstoreApi"
    );

    // Create the client
    let client = PetstoreApi::new("https://petstore3.swagger.io/api/v3");

    // Fetch pets by status - should succeed for "available"
    match client.find_pets_by_status("available").await {
        Ok(pets) => {
            println!("Successfully fetched {} available pets", pets.len());
            // Verify we got a proper response
            assert!(pets.len() > 0, "Expected at least some available pets");
        }
        Err(e) => {
            panic!("Failed to fetch available pets: {:?}", e);
        }
    }
}
