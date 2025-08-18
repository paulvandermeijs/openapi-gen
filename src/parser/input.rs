use syn::{LitStr, Token};

/// Input for the openapi_client macro
pub struct OpenApiInput {
    pub spec_path: String,
    pub client_name: Option<String>,
}

impl syn::parse::Parse for OpenApiInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse first argument (spec path)
        let spec_lit: LitStr = input.parse()?;
        let spec_path = spec_lit.value();

        // Check if there's a second argument (client name)
        let client_name = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let client_lit: LitStr = input.parse()?;
            Some(client_lit.value())
        } else {
            None
        };

        Ok(OpenApiInput {
            spec_path,
            client_name,
        })
    }
}
