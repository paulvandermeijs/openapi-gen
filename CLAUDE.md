# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Note**: For general project information, features, and usage examples, see [README.md](README.md).

## Build and Test Commands

- **Build**: `cargo build`
- **Run tests**: `cargo test`
- **Run specific test**: `cargo test test_name`
- **Check compilation**: `cargo check`
- **Format code**: `cargo fmt`
- **Lint code**: `cargo clippy`

## Architecture Overview

This is a Rust procedural macro crate that generates HTTP API clients from OpenAPI 3.0 specifications. The macro reads OpenAPI JSON/YAML files at compile time and generates complete Rust client code with typed structs and async methods.

### Core Components

- **Procedural macro** (`openapi_client!`): Main entry point that accepts the spec file path as first argument and optional client name as second argument
- **Code generation engine** (`src/lib.rs`): Parses OpenAPI specs and generates:
  - Struct definitions from `components/schemas`
  - Enum types for string enums  
  - HTTP client with async methods for each API endpoint
  - Error handling with `ApiError` enum
- **Generated client features**:
  - Async/await support via `reqwest`
  - JSON serialization via `serde`
  - Automatic path parameter substitution
  - Request body handling
  - Typed response parsing
  - Comprehensive documentation generation:
    - Client struct documentation from API info (title, description, version, license, etc.)
    - Method documentation from operation summaries and descriptions
    - Struct documentation from schema descriptions
    - Field-level documentation for struct properties
  - Rust keyword handling with `r#` prefix

### Usage Pattern

```rust
// Auto-generated client name from API title
openapi_client!("path/to/openapi.json");

// Or with custom client name
openapi_client!("path/to/openapi.json", "MyApiClient");

let client = MyApiClient::new("https://api.example.com");
let result = client.some_endpoint(params).await?;
```

### Dependencies

- `proc-macro2`, `quote`, `syn`: Macro infrastructure
- `openapiv3`: OpenAPI 3.0 specification parsing
- `serde`, `serde_json`, `serde_yaml`: JSON/YAML serialization
- `reqwest`: HTTP client (with JSON feature)
- `thiserror`: Error handling
- `heck`: Case conversions (PascalCase, snake_case)

The crate uses Rust 2024 edition and includes a comprehensive test OpenAPI specification (`openapi.json`) that validates all crate features:

## Test API Features

The included test schema (`openapi.json`) is specifically designed to exercise all crate capabilities:

### API Operations
- **Multiple HTTP methods**: GET, POST, PUT, DELETE
- **Path parameters**: `/users/{userId}`, `/posts/{postId}/comments`
- **Query parameters**: Including Rust keywords (`type`, `self`)
- **Request bodies**: JSON payloads for create/update operations
- **Various response types**: Success, error, and empty responses

### Data Types & Structures
- **All primitive types**: string, integer (int32/int64), number (float/double), boolean
- **Complex objects**: Nested structures with references (`$ref`)
- **Arrays**: Both simple arrays and arrays of objects
- **Enums**: String enumerations with descriptions
- **Type aliases**: Simple type aliases for testing
- **Optional vs required fields**: Mixed field requirements

### Edge Cases & Keyword Handling
- **Rust keywords as field names**: `type`, `self`, `const`
- **Rust keywords as parameter names**: Proper `r#` escaping or `_` suffix
- **Rust keywords in operation IDs**: Tests method name generation
- **Special keyword handling**: `self` → `self_` (cannot be raw identifier)

### Documentation Testing
- **Rich API info**: Title, description, version, license, contact, terms of service
- **Operation documentation**: Summary, description, and metadata
- **Schema documentation**: Object and field descriptions
- **Parameter documentation**: Detailed parameter descriptions

This comprehensive test schema ensures that the generated clients handle real-world API complexity while maintaining type safety and proper documentation.