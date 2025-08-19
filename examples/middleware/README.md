# Custom HTTP Client Example

This example demonstrates how to use the OpenAPI client generator with custom HTTP client configurations for timeouts, user agents, and other reqwest client features.

## What it demonstrates

- Basic client generation and usage
- Custom HTTP client configuration with timeouts and user agents
- The `with_client()` method for using pre-configured HTTP clients
- Error handling with non-existent resources

## Running the example

From the repository root:

```bash
cd examples/middleware
cargo run
```

Or directly:

```bash
cargo run --example middleware
```

## Expected output

The example will:

1. Create a basic client and make a request
2. Create a client with custom timeout and user agent settings
3. Test error handling with a non-existent pet ID
4. Demonstrate the benefits of custom HTTP client configuration

## Key features demonstrated

### Custom Client Configuration
- Custom timeouts for API-specific requirements
- User-agent strings for identification
- HTTP client settings optimization

### Production-ready Patterns
- Error handling strategies
- Timeout configuration
- Client reuse patterns

## Usage patterns

```rust
use std::time::Duration;

// Custom HTTP client
let custom_client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30))
    .user_agent("MyApp/1.0")
    .build()?;

// Use with generated API client
let api = MyApiClient::with_client("https://api.example.com", custom_client);
```

## Common use cases

- Setting API-specific timeouts
- Adding custom headers or user agents  
- Configuring proxies for corporate networks
- Custom TLS/SSL certificate handling
- Connection pooling optimization

## Notes

- Full middleware support (reqwest-middleware) requires additional code generation improvements
- This example focuses on basic reqwest::Client customization
- For complex middleware chains, additional implementation work is needed