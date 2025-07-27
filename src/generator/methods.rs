use openapiv3::ReferenceOr;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use heck::ToSnakeCase;

use crate::utils::create_rust_safe_ident;
use crate::codegen::{ParameterLocation, process_parameter, generate_url_building, reference_or_schema_to_rust_type};
use crate::generator::docs::generate_method_doc_comment;

/// Generate a single API method from an OpenAPI operation
pub fn generate_client_method(
    path: &str,
    http_method: &str,
    operation: &openapiv3::Operation,
) -> Result<TokenStream2, String> {
    let method_name = operation
        .operation_id
        .as_ref()
        .map(|id| {
            let snake_case_id = id.to_snake_case();
            create_rust_safe_ident(&snake_case_id)
        })
        .unwrap_or_else(|| {
            let clean_path = path
                .replace(['{', '}', '/'], "_")
                .trim_matches('_')
                .to_string();
            let method_name = format!("{}_{}", http_method, clean_path);
            create_rust_safe_ident(&method_name)
        });

    let http_method_upper = http_method.to_uppercase();
    let http_method_ident = format_ident!("{}", http_method_upper);

    // Process all parameters
    let mut all_params = Vec::new();

    for param_ref in &operation.parameters {
        let param = match param_ref {
            ReferenceOr::Reference { reference } => {
                return Err(format!("Parameter references not supported: {}", reference));
            }
            ReferenceOr::Item(item) => item,
        };

        let (param_name, param_schema, location) = match param {
            openapiv3::Parameter::Query { parameter_data, .. } => {
                (&parameter_data.name, &parameter_data.format, ParameterLocation::Query)
            }
            openapiv3::Parameter::Path { parameter_data, .. } => {
                (&parameter_data.name, &parameter_data.format, ParameterLocation::Path)
            }
            openapiv3::Parameter::Header { parameter_data, .. } => {
                (&parameter_data.name, &parameter_data.format, ParameterLocation::Header)
            }
            openapiv3::Parameter::Cookie { parameter_data, .. } => {
                (&parameter_data.name, &parameter_data.format, ParameterLocation::Cookie)
            }
        };

        let param_info = process_parameter(param_name, param_schema, location)?;
        all_params.push(param_info);
    }

    // Separate parameters by location
    let path_params: Vec<_> = all_params.iter()
        .filter(|p| p.location == ParameterLocation::Path)
        .collect();
    let query_params: Vec<_> = all_params.iter()
        .filter(|p| p.location == ParameterLocation::Query)
        .collect();

    // Generate parameter list for function signature
    let params = all_params.iter()
        .filter(|p| p.location == ParameterLocation::Path || p.location == ParameterLocation::Query)
        .map(|param| {
            let param_ident = &param.ident;
            let param_type = &param.param_type;
            quote! { #param_ident: #param_type, }
        });

    // Generate URL building code
    let url_building = generate_url_building(path, &path_params, &query_params);

    // Handle request body
    let mut body_param = TokenStream2::new();
    let mut request_building = quote! {
        let mut request = self.client.request(reqwest::Method::#http_method_ident, &url);
    };

    if operation.request_body.is_some() {
        body_param.extend(quote! { body: serde_json::Value, });
        request_building.extend(quote! {
            request = request.json(&body);
        });
    }

    // Determine return type and content type
    let (return_type, content_type) = determine_return_type_from_operation(operation)
        .unwrap_or_else(|| (quote! { () }, "application/json".to_string()));

    // Generate documentation
    let doc_comment = generate_method_doc_comment(operation, path, http_method);

    // Generate response parsing based on content type
    let response_parsing = if content_type.starts_with("text/") {
        quote! {
            if response.status().is_success() {
                let result: String = response.text().await?;
                Ok(result)
            } else {
                Err(ApiError::Api {
                    status: response.status().as_u16(),
                    message: response.text().await.unwrap_or_else(|_| "Unknown error".to_string()),
                })
            }
        }
    } else {
        quote! {
            if response.status().is_success() {
                let result = response.json().await?;
                Ok(result)
            } else {
                Err(ApiError::Api {
                    status: response.status().as_u16(),
                    message: response.text().await.unwrap_or_else(|_| "Unknown error".to_string()),
                })
            }
        }
    };

    Ok(quote! {
        #doc_comment
        pub async fn #method_name(&self, #(#params)* #body_param) -> ApiResult<#return_type> {
            #url_building
            #request_building

            let response = request.send().await?;

            #response_parsing
        }
    })
}

/// Determine the return type and content type from an operation's responses
fn determine_return_type_from_operation(operation: &openapiv3::Operation) -> Option<(TokenStream2, String)> {
    let response_200 = operation
        .responses
        .responses
        .get(&openapiv3::StatusCode::Code(200))?;
    let response = match response_200 {
        ReferenceOr::Reference { .. } => return None,
        ReferenceOr::Item(item) => item,
    };
    
    // Try application/json first - this is the most common case
    if let Some(content) = response.content.get("application/json") {
        if let Some(schema_ref) = content.schema.as_ref() {
            if let Ok(rust_type) = reference_or_schema_to_rust_type(schema_ref) {
                return Some((rust_type, "application/json".to_string()));
            }
        }
    }
    
    // Only try text types if no JSON content was found
    // Try text/plain; charset=utf-8 first (more specific)
    if let Some(_content) = response.content.get("text/plain; charset=utf-8") {
        return Some((quote! { String }, "text/plain; charset=utf-8".to_string()));
    }
    
    // Try text/plain as fallback
    if let Some(_content) = response.content.get("text/plain") {
        return Some((quote! { String }, "text/plain".to_string()));
    }
    
    None
}