use syn::{Ident, LitBool, LitStr, Token};

/// Input for the openapi_client macro
pub struct OpenApiInput {
    pub spec_path: String,
    pub client_name: Option<String>,
    pub use_param_structs: bool,
}

impl syn::parse::Parse for OpenApiInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse first argument (spec path)
        let spec_lit: LitStr = input.parse()?;
        let spec_path = spec_lit.value();

        let mut client_name = None;
        let mut use_param_structs = false;

        // Parse remaining arguments
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;

            // Check if this is a string literal (client name) or an identifier (option key)
            if input.peek(LitStr) {
                // String literal - must be client name
                let client_lit: LitStr = input.parse()?;
                client_name = Some(client_lit.value());
            } else if input.peek(Ident) {
                // Identifier - parse key = value option
                let key: Ident = input.parse()?;
                input.parse::<Token![=]>()?;

                match key.to_string().as_str() {
                    "use_param_structs" => {
                        let value: LitBool = input.parse()?;
                        use_param_structs = value.value;
                    }
                    unknown => {
                        return Err(syn::Error::new_spanned(
                            key,
                            format!("unknown option: {}", unknown),
                        ));
                    }
                }
            } else {
                return Err(syn::Error::new(
                    input.span(),
                    "expected string literal (client name) or identifier (option key)",
                ));
            }
        }

        Ok(OpenApiInput {
            spec_path,
            client_name,
            use_param_structs,
        })
    }
}
