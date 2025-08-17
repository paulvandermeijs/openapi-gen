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
    let structs = generate_structs(&spec)?;
    let client_impl = generate_client_impl(&spec, &client_name)?;
    let error_types = generate_error_types();

    // Generate client documentation
    let client_doc = generate_client_doc_comment(&spec, &client_name.to_string());

    Ok(quote! {
        use serde::{Deserialize, Serialize};
        use std::collections::HashMap;

        #error_types

        #structs

        #client_doc
        #[derive(Clone)]
        pub struct #client_name<C = reqwest::Client> {
            base_url: String,
            client: C,
        }

        #client_impl
    })
}
