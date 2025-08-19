# Petstore API Example

This example demonstrates basic usage of the OpenAPI client generator with the
official Petstore API.

## What it demonstrates

- Basic client generation from a remote OpenAPI specification
- Making API calls (GET, POST)
- Handling JSON request and response bodies
- Error handling for both network and API errors
- Real-world usage patterns

## Running the example

From the repository root:

```bash
cd examples/petstore
cargo run
```

Or directly:

```bash
cargo run --example petstore
```

## Expected output

The example will:

1. Generate a client from the Petstore OpenAPI spec
2. Attempt to add a new pet (may fail due to validation)
3. Fetch available pets from the store
4. Try to retrieve a specific pet (will likely return 404)
5. Demonstrate error handling with an invalid request

## Notes

- The Petstore API is a public demo API, so some operations may fail
- Failures are expected and demonstrate error handling
- The API may have validation rules that cause some requests to fail
- This is a real working example you can modify and experiment with
