# Parameter Structs Example

This example demonstrates the parameter structs feature of the OpenAPI client generator, showing how it improves API ergonomics for methods with multiple parameters.

## What it demonstrates

- Real parameter structs implementation using the local test OpenAPI spec
- Builder pattern with fluent API (`with_*` methods)
- Required vs optional parameter handling
- Default parameter struct creation
- Direct field modification
- Benefits over traditional parameter passing

## Running the example

From the repository root:

```bash
cd examples/param_structs
cargo run
```

Or directly:

```bash
cargo run --example param_structs
```

## Parameter Structs vs Regular Parameters

### Without Parameter Structs
```rust
// Many parameters become unwieldy
client.list_users(Some(10), Some(0), Some("admin".to_string()), None, None).await?;
```

### With Parameter Structs
```rust
// Clean, readable, self-documenting
let params = ListUsersParams::new()
    .with_limit(10)
    .with_offset(0)
    .with_type("admin".to_string());
client.list_users(params).await?;
```

## Key Features Demonstrated

### 1. Builder Pattern
```rust
let params = ListUsersParams::new()
    .with_limit(10)
    .with_offset(0)
    .with_type("admin".to_string());
```

### 2. Required Parameters
```rust
// Required parameters go in new()
let params = GetUserByIdParams::new(123i64);
```

### 3. Default Values
```rust
// All optional parameters default to None
let params: ListUsersParams = Default::default();
```

### 4. Direct Field Access
```rust
let mut params = ListUsersParams::new();
params.limit = Some(50);
params.r#type = Some("guest".to_string());
```

## Benefits of Parameter Structs

- **Clean, readable code** - Named parameters are self-documenting
- **Type safety** - Compile-time validation for all parameters
- **Future-proof** - New optional parameters won't break existing code
- **Consistent patterns** - Uniform API across all methods
- **Flexible usage** - Builder pattern OR direct field access

## When to Use Parameter Structs

Enable parameter structs when:
- APIs have methods with 3+ parameters
- Many parameters are optional
- You want more readable client code
- APIs frequently add new optional parameters
- Working with complex query parameter combinations

## Usage

Enable parameter structs in your macro invocation:

```rust
openapi_client!("openapi.json", "MyClient", use_param_structs = true);
```

## Notes

- This example uses the local test OpenAPI specification (`openapi.json`)
- Parameter structs work best with APIs that have well-defined optional parameters
- The feature generates both builder methods and direct field access
- Required parameters are always passed to the `new()` constructor