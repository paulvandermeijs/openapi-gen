use proc_macro2::TokenStream;
use syn::{Ident, LitBool, LitStr, Token, parenthesized};

/// Input for the openapi_client macro
pub struct OpenApiInput {
    pub spec_path: String,
    pub client_name: Option<String>,
    pub use_param_structs: bool,
    pub struct_attrs: Vec<TokenStream>,
}

impl syn::parse::Parse for OpenApiInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse first argument (spec path)
        let spec_lit: LitStr = input.parse()?;
        let spec_path = spec_lit.value();

        let mut client_name = None;
        let mut use_param_structs = false;
        let mut struct_attrs = Vec::new();

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
                    "struct_attrs" => {
                        // Parse parenthesized list of attribute contents
                        let content;
                        parenthesized!(content in input);

                        // Parse attribute contents as token streams separated by commas
                        while !content.is_empty() {
                            // Parse tokens until we hit a comma or the end
                            let mut attr_tokens = TokenStream::new();

                            // Keep parsing tokens until we hit a comma at the top level
                            let depth = 0;
                            while !content.is_empty() {
                                if content.peek(Token![,]) && depth == 0 {
                                    break;
                                }

                                // Track parenthesis depth
                                if content.peek(syn::token::Paren) {
                                    let group: proc_macro2::Group = content.parse()?;
                                    attr_tokens.extend(std::iter::once(
                                        proc_macro2::TokenTree::Group(group),
                                    ));
                                } else if content.peek(Token![,]) {
                                    break;
                                } else {
                                    // Parse any other token
                                    let token: TokenStream = content.parse()?;
                                    attr_tokens.extend(token);
                                }
                            }

                            struct_attrs.push(attr_tokens);

                            // Consume comma if present
                            if content.peek(Token![,]) {
                                content.parse::<Token![,]>()?;
                            }
                        }
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
            struct_attrs,
        })
    }
}
