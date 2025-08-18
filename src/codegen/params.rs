use heck::ToSnakeCase;
use openapiv3::{ReferenceOr, SchemaKind, Type};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;

use crate::codegen::reference_or_schema_to_rust_type;
use crate::utils::create_rust_safe_ident;

/// Information about a parameter for code generation
pub struct ParameterInfo {
    pub name: String,
    pub ident: Ident,
    pub param_type: TokenStream2,
    pub location: ParameterLocation,
    pub is_array: bool,
    pub required: bool,
}

/// Location where the parameter is used
#[derive(Debug, PartialEq)]
pub enum ParameterLocation {
    Path,
    Query,
    Header,
    Cookie,
}

/// Process a parameter and return its information for code generation
pub fn process_parameter(
    param_name: &str,
    param_schema: &openapiv3::ParameterSchemaOrContent,
    location: ParameterLocation,
    required: bool,
) -> Result<ParameterInfo, String> {
    let snake_case_param = param_name.to_snake_case();
    let param_ident = create_rust_safe_ident(&snake_case_param);

    let base_type = match param_schema {
        openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => {
            // For parameters, convert String types to &str for better ergonomics
            let rust_type = reference_or_schema_to_rust_type(schema_ref)?;
            let type_str = rust_type.to_string();
            if type_str.trim() == "String" {
                quote! { &str }
            } else {
                rust_type
            }
        }
        _ => quote! { &str },
    };

    // Wrap optional parameters in Option<T>
    // Path parameters are always required by OpenAPI spec
    let param_type = if required || location == ParameterLocation::Path {
        base_type
    } else {
        quote! { Option<#base_type> }
    };

    // Check if this is an array parameter
    let is_array = match param_schema {
        openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => match schema_ref {
            ReferenceOr::Item(schema) => {
                matches!(schema.schema_kind, SchemaKind::Type(Type::Array(_)))
            }
            _ => false,
        },
        _ => false,
    };

    Ok(ParameterInfo {
        name: param_name.to_string(),
        ident: param_ident,
        param_type,
        location,
        is_array,
        required,
    })
}

/// Generate URL building code for path and query parameters
pub fn generate_url_building(
    path: &str,
    path_params: &[&ParameterInfo],
    query_params: &[&ParameterInfo],
) -> TokenStream2 {
    let mut url_building = if path_params.is_empty() {
        quote! {
            let mut url = format!("{}{}", self.base_url, #path);
        }
    } else {
        // Handle path parameters
        let path_replacements = path_params.iter().map(|param| {
            let placeholder = format!("{{{}}}", param.name);
            let param_ident = &param.ident;
            quote! {
                .replace(#placeholder, &#param_ident.to_string())
            }
        });

        quote! {
            let mut url = format!("{}{}", self.base_url, #path) #(#path_replacements)*;
        }
    };

    // Add query parameters if any
    if !query_params.is_empty() {
        let query_building = query_params.iter().map(|param| {
            let param_name = &param.name;
            let param_ident = &param.ident;

            // Generate the appropriate value expression
            let value_expr = if param.is_array {
                generate_array_value_expr(param_ident)
            } else {
                generate_single_value_expr(param_ident)
            };

            // Generate the append code
            let append_code = generate_param_append_code(param_name, value_expr);

            // Wrap in optional handling if needed
            if param.required {
                append_code
            } else {
                wrap_optional_code(append_code, param_ident)
            }
        });

        url_building.extend(quote! {
            let mut parsed_url = reqwest::Url::parse(&url).map_err(|e| ApiError::Api {
                status: 400,
                message: format!("Invalid URL: {}", e)
            })?;
            #(#query_building)*
            let url = parsed_url.to_string();
        });
    }

    url_building
}

/// Helper function to generate the core parameter append logic
fn generate_param_append_code(param_name: &str, value_expr: TokenStream2) -> TokenStream2 {
    quote! {
        parsed_url.query_pairs_mut().append_pair(#param_name, &#value_expr);
    }
}

/// Helper function to generate array value expression
fn generate_array_value_expr(param_ident: &Ident) -> TokenStream2 {
    quote! {
        {
            let param_value = #param_ident.iter()
                .map(|n| n.to_string())
                .collect::<Vec<String>>()
                .join(",");
            param_value
        }
    }
}

/// Helper function to generate single value expression
fn generate_single_value_expr(param_ident: &Ident) -> TokenStream2 {
    quote! { #param_ident.to_string() }
}

/// Helper function to wrap code for optional parameters using variable shadowing
fn wrap_optional_code(inner_code: TokenStream2, param_ident: &Ident) -> TokenStream2 {
    quote! {
        if let Some(ref #param_ident) = #param_ident {
            #inner_code
        }
    }
}
