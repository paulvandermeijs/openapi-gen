use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generate error types for the API client
pub fn generate_error_types() -> TokenStream2 {
    let middleware_error = if cfg!(feature = "middleware") {
        quote! {
            /// Middleware error
            #[error("Middleware error: {0}")]
            Middleware(String),
        }
    } else {
        quote! {}
    };

    quote! {
        #[derive(Debug, thiserror::Error)]
        pub enum ApiError {
            #[error("HTTP error: {0}")]
            Http(#[from] reqwest::Error),

            #[error("Serialization error: {0}")]
            Serialization(#[from] serde_json::Error),

            #[error("API error {status}: {message}")]
            Api { status: u16, message: String },

            #middleware_error
        }

        pub type ApiResult<T> = Result<T, ApiError>;
    }
}
