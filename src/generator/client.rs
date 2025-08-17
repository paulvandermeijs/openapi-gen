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

    // Generate middleware implementation only if the feature is enabled
    let middleware_impl = if cfg!(feature = "middleware") {
        quote! {
            impl #client_name<reqwest_middleware::ClientWithMiddleware> {
                async fn send_request(request: reqwest_middleware::RequestBuilder) -> ApiResult<reqwest::Response> {
                    request.send().await.map_err(|e| match e {
                        reqwest_middleware::Error::Reqwest(e) => ApiError::Http(e),
                        e => ApiError::Middleware(e.to_string()),
                    })
                }

                #api_methods
            }
        }
    } else {
        quote! {}
    };

    // Build complete impl block
    Ok(quote! {
        // Default implementation with reqwest::Client
        impl #client_name {
            /// Create a new API client with the specified base URL
            pub fn new(base_url: impl Into<String>) -> Self {
                Self {
                    base_url: base_url.into(),
                    client: reqwest::Client::new(),
                }
            }
        }

        // Generic implementation for any HTTP client
        impl<C> #client_name<C> {
            /// Create a new API client with a custom HTTP client
            pub fn with_client(base_url: impl Into<String>, client: C) -> Self {
                Self {
                    base_url: base_url.into(),
                    client,
                }
            }
        }

        // Helper trait for sending requests
        impl #client_name<reqwest::Client> {
            async fn send_request(request: reqwest::RequestBuilder) -> ApiResult<reqwest::Response> {
                request.send().await.map_err(ApiError::Http)
            }

            #api_methods
        }

        // Helper for middleware client - only generate if middleware feature is enabled
        #middleware_impl

    })
}
