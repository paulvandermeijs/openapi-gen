use openapiv3::{OpenAPI, Operation, Parameter, PathItem, ReferenceOr};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote};

use crate::codegen::params::{ParameterInfo, ParameterLocation};
use crate::codegen::reference_or_schema_to_rust_type;
use crate::utils::create_rust_safe_ident;
use heck::{ToPascalCase, ToSnakeCase};

/// Generate parameter structs for all operations in the OpenAPI spec
pub fn generate_param_structs(spec: &OpenAPI) -> Result<TokenStream2, String> {
    let mut structs = Vec::new();

    for (path, path_item) in &spec.paths.paths {
        if let ReferenceOr::Item(path_item) = path_item {
            generate_structs_for_path(path, path_item, &mut structs)?;
        }
    }

    Ok(quote! {
        #(#structs)*
    })
}

/// Generate parameter structs for all operations in a single path
fn generate_structs_for_path(
    path: &str,
    path_item: &PathItem,
    structs: &mut Vec<TokenStream2>,
) -> Result<(), String> {
    let operations = [
        ("get", &path_item.get),
        ("post", &path_item.post),
        ("put", &path_item.put),
        ("delete", &path_item.delete),
        ("patch", &path_item.patch),
        ("head", &path_item.head),
        ("options", &path_item.options),
        ("trace", &path_item.trace),
    ];

    for (method, operation) in operations {
        if let Some(operation) = operation {
            generate_struct_for_operation(path, method, operation, structs)?;
        }
    }

    Ok(())
}

/// Generate a parameter struct for a single operation
fn generate_struct_for_operation(
    path: &str,
    method: &str,
    operation: &Operation,
    structs: &mut Vec<TokenStream2>,
) -> Result<(), String> {
    // Get operation ID or generate one
    let operation_id = operation
        .operation_id
        .as_ref()
        .cloned()
        .unwrap_or_else(|| generate_operation_id(method, path));

    // Parse all parameters
    let mut params = Vec::new();

    // Parse operation parameters
    for param_ref in &operation.parameters {
        if let ReferenceOr::Item(param) = param_ref {
            match param {
                Parameter::Query { parameter_data, .. } => {
                    let param_info = process_parameter_for_struct(
                        &parameter_data.name,
                        &parameter_data.format,
                        ParameterLocation::Query,
                        parameter_data.required,
                    )?;
                    params.push(param_info);
                }
                Parameter::Header { parameter_data, .. } => {
                    let param_info = process_parameter_for_struct(
                        &parameter_data.name,
                        &parameter_data.format,
                        ParameterLocation::Header,
                        parameter_data.required,
                    )?;
                    params.push(param_info);
                }
                Parameter::Path { parameter_data, .. } => {
                    let param_info = process_parameter_for_struct(
                        &parameter_data.name,
                        &parameter_data.format,
                        ParameterLocation::Path,
                        true, // Path parameters are always required
                    )?;
                    params.push(param_info);
                }
                Parameter::Cookie { parameter_data, .. } => {
                    let param_info = process_parameter_for_struct(
                        &parameter_data.name,
                        &parameter_data.format,
                        ParameterLocation::Cookie,
                        parameter_data.required,
                    )?;
                    params.push(param_info);
                }
            }
        }
    }

    // Only generate struct if there are parameters
    if !params.is_empty() {
        let struct_name = format_ident!("{}Params", operation_id.to_pascal_case());
        let struct_def = generate_param_struct(&struct_name, &params)?;
        structs.push(struct_def);
    }

    Ok(())
}

/// Process a parameter for use in parameter structs (uses String instead of &str)
fn process_parameter_for_struct(
    param_name: &str,
    param_schema: &openapiv3::ParameterSchemaOrContent,
    location: ParameterLocation,
    required: bool,
) -> Result<ParameterInfo, String> {
    let snake_case_param = param_name.to_snake_case();
    let param_ident = create_rust_safe_ident(&snake_case_param);

    let base_type = match param_schema {
        openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => {
            // For parameter structs, always use String instead of &str to avoid lifetimes
            let rust_type = reference_or_schema_to_rust_type(schema_ref)?;
            let type_str = rust_type.to_string();
            if type_str.trim() == "String" || type_str.contains("& str") {
                quote! { String }
            } else {
                rust_type
            }
        }
        _ => quote! { String },
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
                matches!(
                    schema.schema_kind,
                    openapiv3::SchemaKind::Type(openapiv3::Type::Array(_))
                )
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

/// Generate the actual parameter struct code
fn generate_param_struct(
    struct_name: &Ident,
    params: &[ParameterInfo],
) -> Result<TokenStream2, String> {
    // Separate required and optional parameters
    let required_params: Vec<_> = params.iter().filter(|p| p.required).collect();
    let optional_params: Vec<_> = params.iter().filter(|p| !p.required).collect();

    // Generate struct fields
    let fields = params.iter().map(|param| {
        let name = &param.ident;
        let param_type = &param.param_type;
        quote! {
            pub #name: #param_type,
        }
    });

    // Generate constructor
    let constructor = generate_constructor(&required_params, &optional_params);

    // Generate builder methods for optional parameters
    let builder_methods = generate_builder_methods(&optional_params);

    // Generate Default implementation if no required parameters
    let default_impl = if required_params.is_empty() {
        quote! {
            impl Default for #struct_name {
                fn default() -> Self {
                    Self::new()
                }
            }
        }
    } else {
        quote! {}
    };

    // For parameter structs, we use String instead of &str to avoid lifetime complexity
    // This makes the API more ergonomic and avoids lifetime propagation issues

    Ok(quote! {
        pub struct #struct_name {
            #(#fields)*
        }

        impl #struct_name {
            #constructor
            #(#builder_methods)*
        }

        #default_impl
    })
}

/// Generate constructor method
fn generate_constructor(
    required_params: &[&ParameterInfo],
    optional_params: &[&ParameterInfo],
) -> TokenStream2 {
    if required_params.is_empty() {
        // No required parameters - simple constructor
        let optional_init = optional_params.iter().map(|param| {
            let name = &param.ident;
            quote! { #name: None, }
        });

        quote! {
            pub fn new() -> Self {
                Self {
                    #(#optional_init)*
                }
            }
        }
    } else {
        // Has required parameters - constructor with args
        let required_args = required_params.iter().map(|param| {
            let name = &param.ident;
            // For required parameters, use the actual type without Option wrapper
            let param_type = &param.param_type;
            quote! { #name: #param_type, }
        });

        let required_init = required_params.iter().map(|param| {
            let name = &param.ident;
            quote! { #name, }
        });

        let optional_init = optional_params.iter().map(|param| {
            let name = &param.ident;
            quote! { #name: None, }
        });

        quote! {
            pub fn new(#(#required_args)*) -> Self {
                Self {
                    #(#required_init)*
                    #(#optional_init)*
                }
            }
        }
    }
}

/// Generate builder methods for optional parameters
fn generate_builder_methods(optional_params: &[&ParameterInfo]) -> Vec<TokenStream2> {
    optional_params
        .iter()
        .map(|param| {
            let param_name = &param.ident;
            let method_name = format_ident!("with_{}", param_name);

            // Extract inner type from Option<T>
            let inner_type = if param.param_type.to_string().contains("Option <") {
                let type_str = param.param_type.to_string();
                let inner = type_str
                    .strip_prefix("Option < ")
                    .and_then(|s| s.strip_suffix(" >"))
                    .unwrap_or(&type_str);
                syn::parse_str::<syn::Type>(inner).unwrap()
            } else {
                syn::parse2(param.param_type.clone()).unwrap()
            };

            // For String parameters, accept both &str and String for convenience
            let input_type = if inner_type.to_token_stream().to_string() == "String" {
                quote! { impl Into<String> }
            } else {
                quote! { #inner_type }
            };

            let assignment = if inner_type.to_token_stream().to_string() == "String" {
                quote! { self.#param_name = Some(#param_name.into()); }
            } else {
                quote! { self.#param_name = Some(#param_name); }
            };

            quote! {
                pub fn #method_name(mut self, #param_name: #input_type) -> Self {
                    #assignment
                    self
                }
            }
        })
        .collect()
}

/// Generate operation ID from method and path
fn generate_operation_id(method: &str, path: &str) -> String {
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
