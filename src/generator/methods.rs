use heck::{ToPascalCase, ToSnakeCase};
use openapiv3::ReferenceOr;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::codegen::{
    ParameterLocation, generate_url_building, process_parameter, reference_or_schema_to_rust_type,
};
use crate::generator::docs::generate_method_doc_comment;
use crate::utils::create_rust_safe_ident;

/// Generate a single API method from an OpenAPI operation
pub fn generate_client_method(
    path: &str,
    http_method: &str,
    operation: &openapiv3::Operation,
    use_param_structs: bool,
) -> Result<TokenStream2, String> {
    generate_client_method_with_mode(path, http_method, operation, false, use_param_structs)
}

/// Generate a blocking API method from an OpenAPI operation
pub fn generate_blocking_client_method(
    path: &str,
    http_method: &str,
    operation: &openapiv3::Operation,
    use_param_structs: bool,
) -> Result<TokenStream2, String> {
    generate_client_method_with_mode(path, http_method, operation, true, use_param_structs)
}

/// Generate a single API method from an OpenAPI operation with async/blocking mode
fn generate_client_method_with_mode(
    path: &str,
    http_method: &str,
    operation: &openapiv3::Operation,
    is_blocking: bool,
    use_param_structs: bool,
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

        let (param_name, param_schema, location, required) = match param {
            openapiv3::Parameter::Query { parameter_data, .. } => (
                &parameter_data.name,
                &parameter_data.format,
                ParameterLocation::Query,
                parameter_data.required,
            ),
            openapiv3::Parameter::Path { parameter_data, .. } => (
                &parameter_data.name,
                &parameter_data.format,
                ParameterLocation::Path,
                parameter_data.required,
            ),
            openapiv3::Parameter::Header { parameter_data, .. } => (
                &parameter_data.name,
                &parameter_data.format,
                ParameterLocation::Header,
                parameter_data.required,
            ),
            openapiv3::Parameter::Cookie { parameter_data, .. } => (
                &parameter_data.name,
                &parameter_data.format,
                ParameterLocation::Cookie,
                parameter_data.required,
            ),
        };

        let param_info = process_parameter(param_name, param_schema, location, required)?;
        all_params.push(param_info);
    }

    // Separate parameters by location
    let path_params: Vec<_> = all_params
        .iter()
        .filter(|p| p.location == ParameterLocation::Path)
        .collect();
    let query_params: Vec<_> = all_params
        .iter()
        .filter(|p| p.location == ParameterLocation::Query)
        .collect();

    // Generate parameter list for function signature
    let (params, param_access_code) = if use_param_structs {
        // Use parameter struct approach
        let method_params: Vec<_> = all_params
            .iter()
            .filter(|p| {
                p.location == ParameterLocation::Path || p.location == ParameterLocation::Query
            })
            .collect();

        if method_params.is_empty() {
            // No parameters - keep empty signature
            (quote! {}, quote! {})
        } else {
            // Generate parameter struct name
            let operation_id = operation
                .operation_id
                .as_ref()
                .cloned()
                .unwrap_or_else(|| generate_operation_id_for_struct(http_method, path));
            let struct_name = format_ident!("{}Params", operation_id.to_pascal_case());

            // Method signature uses parameter struct
            let params = quote! { params: #struct_name, };

            // Code to extract values from parameter struct
            let param_extractions = method_params.iter().map(|param| {
                let field_name = &param.ident;
                let var_name = format_ident!("{}_value", field_name);
                quote! {
                    let #var_name = params.#field_name;
                }
            });

            let param_access_code = quote! {
                #(#param_extractions)*
            };

            (params, param_access_code)
        }
    } else {
        // Use individual parameters approach (existing behavior)
        let params = all_params
            .iter()
            .filter(|p| {
                p.location == ParameterLocation::Path || p.location == ParameterLocation::Query
            })
            .map(|param| {
                let param_ident = &param.ident;
                let param_type = &param.param_type;
                quote! { #param_ident: #param_type, }
            });
        (quote! { #(#params)* }, quote! {})
    };

    // Generate URL building code
    let url_building = if use_param_structs {
        generate_url_building_with_param_structs(path, &path_params, &query_params)
    } else {
        generate_url_building(path, &path_params, &query_params)
    };

    // Handle request body
    let mut body_param = TokenStream2::new();
    let mut request_building = quote! {
        let parsed_url = reqwest::Url::parse(&url).map_err(|e| ApiError::Api {
            status: 400,
            message: format!("Invalid URL: {}", e)
        })?;
        let mut request = self.client.request(reqwest::Method::#http_method_ident, parsed_url);
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
        if is_blocking {
            quote! {
                if response.status().is_success() {
                    let result: String = response.text()?;
                    Ok(result)
                } else {
                    Err(ApiError::Api {
                        status: response.status().as_u16(),
                        message: response.text().unwrap_or_else(|_| "Unknown error".to_string()),
                    })
                }
            }
        } else {
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
        }
    } else {
        if is_blocking {
            quote! {
                if response.status().is_success() {
                    let result = response.json()?;
                    Ok(result)
                } else {
                    Err(ApiError::Api {
                        status: response.status().as_u16(),
                        message: response.text().unwrap_or_else(|_| "Unknown error".to_string()),
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
        }
    };

    let (signature, send_call) = if is_blocking {
        (
            quote! { pub fn #method_name(&self, #params #body_param) -> ApiResult<#return_type> },
            quote! { let response = Self::send_request(request)?; },
        )
    } else {
        (
            quote! { pub async fn #method_name(&self, #params #body_param) -> ApiResult<#return_type> },
            quote! { let response = Self::send_request(request).await?; },
        )
    };

    Ok(quote! {
        #doc_comment
        #signature {
            #param_access_code
            #url_building
            #request_building

            #send_call

            #response_parsing
        }
    })
}

/// Determine the return type and content type from an operation's responses
fn determine_return_type_from_operation(
    operation: &openapiv3::Operation,
) -> Option<(TokenStream2, String)> {
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

/// Generate operation ID from method and path (for parameter struct naming)
fn generate_operation_id_for_struct(method: &str, path: &str) -> String {
    // Convert path to camelCase operation name
    let path_parts: Vec<&str> = path
        .split('/')
        .filter(|s| !s.is_empty() && !s.starts_with('{'))
        .collect();

    if path_parts.is_empty() {
        method.to_string()
    } else {
        format!("{}{}", method, path_parts.join("_").to_pascal_case())
    }
}

/// Generate URL building code when using parameter structs
/// This is similar to generate_url_building but uses the extracted _value variables
fn generate_url_building_with_param_structs(
    path: &str,
    path_params: &[&crate::codegen::params::ParameterInfo],
    query_params: &[&crate::codegen::params::ParameterInfo],
) -> TokenStream2 {
    let mut url_building = if path_params.is_empty() {
        quote! {
            let mut url = format!("{}{}", self.base_url, #path);
        }
    } else {
        // Handle path parameters using extracted values
        let path_replacements = path_params.iter().map(|param| {
            let param_name = &param.name;
            let var_name = format_ident!("{}_value", param.ident);
            let placeholder = format!("{{{}}}", param_name);
            quote! {
                path = path.replace(#placeholder, &#var_name.to_string());
            }
        });

        quote! {
            let mut path = #path.to_string();
            #(#path_replacements)*
            let mut url = format!("{}{}", self.base_url, path);
        }
    };

    // Handle query parameters using extracted values
    if !query_params.is_empty() {
        let query_building = query_params.iter().map(|param| {
            let param_name = &param.name;
            let var_name = format_ident!("{}_value", param.ident);

            // Define the formatting expression once for both required and optional
            let formatting_expr = if param.is_array {
                quote! { #var_name.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(",") }
            } else {
                quote! { #var_name.to_string() }
            };

            // Common code for appending the parameter
            let append_param = quote! {
                let formatted_value = #formatting_expr;
                url.push_str(&format!("{}{}={}", if url.contains('?') { "&" } else { "?" }, #param_name, formatted_value));
            };

            if param.required {
                // For required params, use the value directly
                append_param
            } else {
                // For optional params, shadow the variable name after unwrapping
                quote! {
                    if let Some(#var_name) = &#var_name {
                        #append_param
                    }
                }
            }
        });

        url_building.extend(quote! {
            #(#query_building)*
        });
    }

    url_building
}
