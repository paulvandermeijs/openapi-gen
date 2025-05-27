# OpenAPI Client Generator

A Rust procedural macro crate that generates type-safe, async HTTP clients from
OpenAPI 3.0 specifications.

## Features

- 🚀 **Zero-runtime dependencies** - Pure compile-time code generation
- 🔒 **Type-safe** - Full Rust type system integration with proper error
  handling
- 📚 **Auto-documented** - Generates comprehensive documentation from OpenAPI
  descriptions
- 🛡️ **Keyword-safe** - Handles Rust keywords automatically with proper escaping
- ⚡ **Async/await** - Built on `reqwest` with full async support
- 🎯 **Easy to use** - Simple macro interface with sensible defaults
- 🔧 **Flexible** - Supports both JSON and YAML OpenAPI specifications

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
openapi-gen = "0.3.0"
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
```

### Basic Usage

```rust
use openapi_gen::openapi_client;

// Generate client from OpenAPI spec with auto-generated name
openapi_client!("path/to/your/openapi.json");

// Or specify a custom client name
openapi_client!("path/to/your/openapi.json", "MyApiClient");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client instance
    let client = MyApiClient::new("https://api.example.com");

    // Use the generated methods
    let users = client.list_users(Some(10), None, None).await?;
    let user = client.get_user_by_id(123).await?;

    Ok(())
}
```

## Generated Code

The macro generates:

### 1. Type-Safe Structs

```rust
/// Represents a user in the system with comprehensive profile information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier for the user
    pub id: i64,
    /// User's chosen username
    pub username: String,
    /// User's email address
    pub email: String,
    /// User type (tests Rust keyword as field name)
    pub r#type: String,
    // ... more fields
}
```

### 2. Async Client Methods

```rust
impl MyApiClient {
    /// List all users
    ///
    /// Retrieve a paginated list of all users in the system. Supports filtering and sorting.
    ///
    /// **HTTP Method:** `GET`
    /// **Path:** `/users`
    /// **Operation ID:** `listUsers`
    pub async fn list_users(
        &self,
        limit: Option<i32>,
        offset: Option<i64>,
        r#type: Option<String>
    ) -> ApiResult<UserList> {
        // Generated implementation
    }
}
```

### 3. Comprehensive Documentation

The generated client includes:

- **API information** from the OpenAPI `info` section
- **Method documentation** from operation summaries and descriptions
- **Type documentation** from schema descriptions
- **Field documentation** from property descriptions

## OpenAPI Feature Support

| Feature               | Support | Notes                                                |
| --------------------- | ------- | ---------------------------------------------------- |
| **HTTP Methods**      | ✅      | GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, TRACE  |
| **Path Parameters**   | ✅      | Automatic URL template substitution                  |
| **Query Parameters**  | ✅      | Optional and required parameters                     |
| **Request Bodies**    | ✅      | JSON request bodies                                  |
| **Response Types**    | ✅      | Typed response parsing                               |
| **Schema References** | ✅      | `$ref` resolution for reusable components            |
| **Enums**             | ✅      | String enumerations with proper Rust enum generation |
| **Arrays**            | ✅      | `Vec<T>` generation for array types                  |
| **Objects**           | ✅      | Struct generation with proper field types            |
| **Optional Fields**   | ✅      | `Option<T>` for non-required fields                  |
| **Nested Objects**    | ✅      | Complex object hierarchies                           |
| **Type Aliases**      | ✅      | Simple type aliases                                  |
| **Rust Keywords**     | ✅      | Automatic escaping with `r#` or `_` suffix           |

## Rust Keyword Handling

The generator automatically handles Rust keywords in:

- **Field names**: `type` → `r#type`, `self` → `self_`
- **Parameter names**: `const` → `r#const`
- **Method names**: Derived from operation IDs with keyword escaping

Special handling for `self` and `Self` (cannot be raw identifiers):

- `self` → `self_`
- `Self` → `Self_`

## Error Handling

The generated client includes a comprehensive error type:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },
}

pub type ApiResult<T> = Result<T, ApiError>;
```

## Configuration

### Client Customization

```rust
// Basic client
let client = MyApiClient::new("https://api.example.com");

// Client with custom HTTP client
let http_client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .build()?;
let client = MyApiClient::with_client("https://api.example.com", http_client);
```

## Examples

### Complete Example

```rust
use openapi_gen::openapi_client;
use serde_json::json;

// Generate the client
openapi_client!("openapi.json", "TestApi");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TestApi::new("https://api.test.com/v2");

    // List users with pagination
    let users = client.list_users(Some(20), Some(0), None).await?;
    println!("Found {} users", users.total);

    // Create a new user
    let new_user = json!({
        "username": "john_doe",
        "email": "john@example.com",
        "firstName": "John",
        "lastName": "Doe"
    });
    let created_user = client.create_user(new_user).await?;

    // Get user details
    let user = client.get_user_by_id(created_user.id).await?;
    println!("User: {} <{}>", user.username, user.email);

    // Update user
    let update_data = json!({
        "firstName": "Jonathan"
    });
    let updated_user = client.update_user(user.id, update_data).await?;

    Ok(())
}
```

## Requirements

- **Rust**: 2024 edition or later
- **OpenAPI**: 3.0.x specifications (JSON or YAML)

## Dependencies

**Runtime dependencies** (required in your project):
- `reqwest` - HTTP client with JSON support
- `serde` - Serialization framework (with derive feature)
- `serde_json` - JSON serialization
- `thiserror` - Error handling
- `tokio` - Async runtime

**Build-time dependencies** (used by the macro):
- `proc-macro2`, `quote`, `syn` - Procedural macro infrastructure
- `openapiv3` - OpenAPI 3.0 specification parsing
- `serde_yaml` - YAML parsing for specs
- `heck` - Case conversions
- `tokio` - For compile-time URL fetching

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Documentation

```bash
cargo doc --no-deps --open
```

The project includes a comprehensive test OpenAPI specification that validates
all crate features.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file
for details.
