use openapiv3::{ReferenceOr, SchemaKind, Type};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use heck::ToSnakeCase;

use crate::utils::create_rust_safe_ident;
use crate::codegen::reference_or_schema_to_rust_type;

/// Information about a parameter for code generation
pub struct ParameterInfo {
    pub name: String,
    pub ident: Ident,
    pub param_type: TokenStream2,
    pub location: ParameterLocation,
    pub is_array: bool,
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
) -> Result<ParameterInfo, String> {
    let snake_case_param = param_name.to_snake_case();
    let param_ident = create_rust_safe_ident(&snake_case_param);

    let param_type = match param_schema {
        openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => {
            reference_or_schema_to_rust_type(schema_ref)?
        }
        _ => quote! { String },
    };

    // Check if this is an array parameter
    let is_array = match param_schema {
        openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => {
            match schema_ref {
                ReferenceOr::Item(schema) => {
                    matches!(schema.schema_kind, SchemaKind::Type(Type::Array(_)))
                }
                _ => false,
            }
        }
        _ => false,
    };

    Ok(ParameterInfo {
        name: param_name.to_string(),
        ident: param_ident,
        param_type,
        location,
        is_array,
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
            
            if param.is_array {
                quote! {
                    // Handle array parameters by joining with commas
                    let param_value = #param_ident.iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<String>>()
                        .join(",");
                    parsed_url.query_pairs_mut().append_pair(#param_name, &param_value);
                }
            } else {
                quote! {
                    // Handle single value parameters
                    parsed_url.query_pairs_mut().append_pair(#param_name, &#param_ident.to_string());
                }
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