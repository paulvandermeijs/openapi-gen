//! Parameter Structs Example
//!
//! This example demonstrates the parameter structs feature, which provides:
//! - Ergonomic parameter handling with named fields
//! - Builder pattern for optional parameters
//! - Better code readability and maintainability
//! - Future-proof API evolution (new optional params don't break existing code)

use openapi_gen::openapi_client;

// Generate client with parameter structs enabled
// This creates a struct for each API method's parameters instead of individual arguments
openapi_client!(
    "https://petstore3.swagger.io/api/v3/openapi.json",
    "PetstoreApi",
    use_param_structs = true
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PetstoreApi::new("https://petstore3.swagger.io/api/v3");
    
    // Example 1: Basic parameter struct usage
    // Instead of: client.find_pets_by_status("available")
    // We use a parameter struct for better clarity
    let status_params = FindPetsByStatusParams::new("available".to_string());
    
    match client.find_pets_by_status(status_params).await {
        Ok(pets) => println!("Found {} available pets", pets.len()),
        Err(e) => eprintln!("Error: {}", e),
    }
    
    // Example 2: Builder pattern with optional parameters
    // The builder pattern makes it clear which parameters are being set
    // and allows for fluent, readable API calls
    let login_params = LoginUserParams::new()
        .with_username("testuser".to_string())
        .with_password("password123".to_string());
    
    // All optional parameters have with_* methods for the builder pattern
    match client.login_user(login_params).await {
        Ok(response) => println!("Login response: {}", response),
        Err(e) => eprintln!("Login failed: {}", e),
    }
    
    // Example 3: Mixed required and optional parameters
    // Required parameters go in new(), optional ones use with_* methods
    let update_params = UpdatePetWithFormParams::new(123)  // pet_id is required
        .with_name("UpdatedPet".to_string())              // name is optional
        .with_status("sold".to_string());                 // status is optional
    
    // The struct fields are accessible if you need to inspect them
    // update_params.pet_id -> i64
    // update_params.name -> Option<String>
    // update_params.status -> Option<String>
    
    match client.update_pet_with_form(update_params).await {
        Ok(_) => println!("Pet updated successfully"),
        Err(e) => eprintln!("Update failed: {}", e),
    }
    
    // Example 4: Direct struct construction
    // You can also construct parameter structs directly
    let mut params = FindPetsByTagsParams::new(vec!["dog".to_string(), "cute".to_string()]);
    
    // And modify fields after construction
    params.tags.push("friendly".to_string());
    
    match client.find_pets_by_tags(params).await {
        Ok(pets) => println!("Found {} pets with specified tags", pets.len()),
        Err(e) => eprintln!("Search failed: {}", e),
    }
    
    // Comparison with regular parameters:
    //
    // Without parameter structs:
    //   client.some_method(param1, param2, Some(param3), None, Some(param5))
    //   - Unclear what each parameter represents
    //   - Easy to mix up parameter order
    //   - Many None values for unused optional parameters
    //
    // With parameter structs:
    //   client.some_method(
    //       SomeMethodParams::new(param1, param2)
    //           .with_param3(param3)
    //           .with_param5(param5)
    //   )
    //   - Clear, named parameters
    //   - No need to specify None for unused optionals
    //   - Impossible to mix up parameter order
    
    Ok(())
}