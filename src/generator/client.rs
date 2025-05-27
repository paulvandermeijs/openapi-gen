use openapiv3::{OpenAPI, ReferenceOr};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;

use crate::generator::methods::generate_client_method;

/// Generate the complete client implementation
pub fn generate_client_impl(spec: &OpenAPI, client_name: &Ident) -> Result<TokenStream2, String> {
    let mut api_methods = TokenStream2::new();

    // Generate methods from paths
    for (path, path_item_ref) in spec.paths.iter() {
        let path_item = match path_item_ref {
            ReferenceOr::Reference { reference } => {
                return Err(format!("Path item references not supported: {}", reference));
            }
            ReferenceOr::Item(item) => item,
        };

        for (method, operation) in [
            ("get", &path_item.get),
            ("post", &path_item.post),
            ("put", &path_item.put),
            ("delete", &path_item.delete),
            ("patch", &path_item.patch),
            ("head", &path_item.head),
            ("options", &path_item.options),
            ("trace", &path_item.trace),
        ] {
            if let Some(op) = operation {
                let method_tokens = generate_client_method(path, method, op)?;
                api_methods.extend(method_tokens);
            }
        }
    }

    // Build complete impl block
    Ok(quote! {
        impl #client_name {
            /// Create a new API client with the specified base URL
            pub fn new(base_url: impl Into<String>) -> Self {
                Self {
                    base_url: base_url.into(),
                    client: reqwest::Client::new(),
                }
            }

            /// Create a new API client with a custom HTTP client
            pub fn with_client(base_url: impl Into<String>, client: reqwest::Client) -> Self {
                Self {
                    base_url: base_url.into(),
                    client,
                }
            }

            #api_methods
        }
    })
}