//! # OpenAPI Client Generator
//!
//! A Rust procedural macro crate that generates type-safe, async HTTP clients from OpenAPI 3.0 specifications.
//!
//! ## Features
//!
//! - **Zero-runtime dependencies** - Pure compile-time code generation
//! - **Type-safe** - Full Rust type system integration with proper error handling
//! - **Auto-documented** - Generates comprehensive documentation from OpenAPI descriptions
//! - **Async/await** - Built on `reqwest` with full async support
//! - **Parameter structs** - Optional ergonomic parameter handling for complex APIs
//! - **Middleware support** - Optional `reqwest-middleware` integration
//! - **Blocking client support** - Optional synchronous HTTP client generation
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! openapi-gen = "0.3"
//! reqwest = { version = "0.12", features = ["json"] }
//! tokio = { version = "1.0", features = ["full"] }
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! thiserror = "1.0"
//! ```
//!
//! ## Usage
//!
//! See the [`openapi_client!`] macro documentation for detailed usage examples.
//!
//! For comprehensive documentation, examples, and advanced features,
//! see the [README](https://github.com/paulvandermeijs/openapi-gen#readme).
//!
//! ## Optional Features
//!
//! - `middleware` - Enables `reqwest-middleware` support for advanced HTTP client features
//! - `blocking` - Generates synchronous HTTP clients using `reqwest::blocking`

mod codegen;
mod generator;
mod parser;
mod utils;

use heck::ToPascalCase;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse_macro_input;

use generator::*;
use parser::*;

/// Generates an API client and structs from an OpenAPI specification
///
/// Supports loading OpenAPI specifications from both local files and remote URLs.
/// The specification format (JSON/YAML) is auto-detected from the file extension
/// or URL path.
///
/// Usage:
/// ```rust,ignore
/// use openapi_gen::openapi_client;
///
/// // From local file with auto-generated client name (derived from API title + "Api")
/// openapi_client!("path/to/openapi.json");
/// openapi_client!("path/to/openapi.yaml");
///
/// // From URL with auto-generated client name
/// openapi_client!("https://api.example.com/openapi.json");
/// openapi_client!("https://raw.githubusercontent.com/user/repo/main/openapi.yaml");
///
/// // With custom client name (works for both files and URLs)
/// openapi_client!("path/to/openapi.json", "MyApiClient");
/// openapi_client!("https://api.example.com/openapi.json", "MyApiClient");
/// ```
#[proc_macro]
pub fn openapi_client(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as OpenApiInput);

    match generate_client(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => syn::Error::new(Span::call_site(), e)
            .to_compile_error()
            .into(),
    }
}

fn generate_client(input: &OpenApiInput) -> Result<TokenStream2, String> {
    // Load and parse the OpenAPI specification
    let spec = load_openapi_spec(input)?;

    let client_name = if let Some(name) = &input.client_name {
        format_ident!("{}", name)
    } else {
        // Derive client name from API title
        let title = spec.info.title.clone();
        let sanitized_title = title
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .to_pascal_case();
        format_ident!("{}Api", sanitized_title)
    };

    // Generate components
    let structs = generate_structs(&spec, &input.struct_attrs)?;
    let client_impl = generate_client_impl(&spec, &client_name, input.use_param_structs)?;
    let error_types = generate_error_types();

    // Generate parameter structs if requested
    let param_structs = if input.use_param_structs {
        generate_param_structs(&spec, &input.struct_attrs)?
    } else {
        quote! {}
    };

    // Generate client documentation
    let client_doc = generate_client_doc_comment(&spec, &client_name.to_string());

    Ok(quote! {
        use serde::{Deserialize, Serialize};
        use std::collections::HashMap;

        #error_types

        #structs

        #param_structs

        #client_doc
        #[derive(Clone)]
        pub struct #client_name<C = reqwest::Client> {
            base_url: String,
            client: C,
        }

        #client_impl
    })
}
