//! Petstore API Client Example
//!
//! This example demonstrates basic usage of the generated API client
//! with the Swagger Petstore API, showing how to:
//! - Create a client instance
//! - Make various API calls (GET, POST)
//! - Handle responses and errors
//! - Work with JSON request/response bodies

use openapi_gen::openapi_client;
use serde_json::json;

// Generate a type-safe client from the official Petstore OpenAPI specification
// This creates all the necessary structs and methods at compile time
openapi_client!(
    "https://petstore3.swagger.io/api/v3/openapi.json",
    "PetstoreApi"
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the API client with the base URL
    // The client is reusable and can be cloned/shared across async tasks
    let client = PetstoreApi::new("https://petstore3.swagger.io/api/v3");

    // Add a new pet to the store
    // The API expects a JSON object conforming to the Pet schema
    let new_pet = json!({
        "id": 12345,
        "name": "Buddy",
        "category": {
            "id": 1,
            "name": "Dogs"
        },
        "photoUrls": ["https://example.com/buddy.jpg"],
        "tags": [
            {
                "id": 1,
                "name": "friendly"
            }
        ],
        "status": "available"
    });

    // POST request to add a pet
    // Returns a strongly-typed Pet struct on success
    match client.add_pet(new_pet).await {
        Ok(pet) => {
            println!("Added pet: {} (ID: {:?})", pet.name, pet.id);
        }
        Err(e) => {
            // API errors are properly typed and include status codes
            eprintln!("Failed to add pet: {}", e);
        }
    }

    // Query pets by their status
    // The status parameter is type-checked at compile time
    match client.find_pets_by_status("available").await {
        Ok(pets) => {
            println!("Found {} available pets", pets.len());
            // The pets vector contains fully typed Pet structs
            for pet in pets.iter().take(3) {
                println!("  - {}", pet.name);
            }
        }
        Err(e) => {
            eprintln!("Failed to find pets: {}", e);
        }
    }

    // Fetch a specific pet by ID
    // This demonstrates path parameter handling
    match client.get_pet_by_id(12345).await {
        Ok(pet) => {
            println!("Retrieved pet: {}", pet.name);
        }
        Err(e) => {
            // 404 errors are expected if the pet doesn't exist
            // The error type includes the HTTP status code
            eprintln!("Pet not found: {}", e);
        }
    }

    // Error handling example with invalid input
    // The API validates the status parameter server-side
    match client.find_pets_by_status("invalid_status").await {
        Ok(pets) => {
            // Some APIs might return an empty list for invalid filters
            println!("Found {} pets", pets.len());
        }
        Err(e) => {
            // Most APIs will return a 400 Bad Request for invalid parameters
            eprintln!("Invalid status parameter: {}", e);
        }
    }

    Ok(())
}
