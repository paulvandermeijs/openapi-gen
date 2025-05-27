use heck::{ToPascalCase, ToSnakeCase};
use openapiv3::{
    ObjectType, OpenAPI, ReferenceOr, Schema, SchemaData, SchemaKind, StringType, Type,
};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::{LitStr, parse_macro_input};

/// Generates an API client and structs from an OpenAPI specification
///
/// Supports loading OpenAPI specifications from both local files and remote URLs.
/// The specification format (JSON/YAML) is auto-detected from the file extension
/// or URL path.
///
/// Usage:
/// ```rust,ignore
/// use openapi_gen::openapi_client;
///
/// // From local file with auto-generated client name (derived from API title + "Api")
/// openapi_client!("path/to/openapi.json");
/// openapi_client!("path/to/openapi.yaml");
///
/// // From URL with auto-generated client name
/// openapi_client!("https://api.example.com/openapi.json");
/// openapi_client!("https://raw.githubusercontent.com/user/repo/main/openapi.yaml");
///
/// // With custom client name (works for both files and URLs)
/// openapi_client!("path/to/openapi.json", "MyApiClient");
/// openapi_client!("https://api.example.com/openapi.json", "MyApiClient");
/// ```
#[proc_macro]
pub fn openapi_client(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as OpenApiInput);

    match generate_client(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => syn::Error::new(Span::call_site(), e)
            .to_compile_error()
            .into(),
    }
}

struct OpenApiInput {
    spec_path: String,
    client_name: Option<String>,
}

impl syn::parse::Parse for OpenApiInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse first argument (spec path)
        let spec_lit: LitStr = input.parse()?;
        let spec_path = spec_lit.value();

        // Check if there's a second argument (client name)
        let client_name = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
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

fn generate_client(input: &OpenApiInput) -> Result<TokenStream2, String> {
    // Read and parse the OpenAPI spec from file or URL
    let spec_content = if is_url(&input.spec_path) {
        fetch_url_content(&input.spec_path)?
    } else {
        std::fs::read_to_string(&input.spec_path)
            .map_err(|e| format!("Failed to read spec file: {}", e))?
    };

    let spec: OpenAPI = if is_yaml_format(&input.spec_path) {
        serde_yaml::from_str(&spec_content).map_err(|e| format!("Failed to parse YAML: {}", e))?
    } else {
        serde_json::from_str(&spec_content).map_err(|e| format!("Failed to parse JSON: {}", e))?
    };

    let client_name = if let Some(name) = &input.client_name {
        format_ident!("{}", name)
    } else {
        // Derive client name from API title
        let title = spec.info.title.clone();
        let sanitized_title = title
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .to_pascal_case();
        format_ident!("{}Api", sanitized_title)
    };

    // Generate components
    let structs = generate_structs(&spec)?;
    let client_impl = generate_client_impl(&spec, &client_name)?;
    let error_types = generate_error_types();
    
    // Generate client documentation
    let client_doc = generate_client_doc_comment(&spec, &client_name.to_string());

    Ok(quote! {
        use serde::{Deserialize, Serialize};
        use std::collections::HashMap;

        #error_types

        #structs

        #client_doc
        #[derive(Clone)]
        pub struct #client_name {
            base_url: String,
            client: reqwest::Client,
        }

        #client_impl
    })
}

fn generate_error_types() -> TokenStream2 {
    quote! {
        #[derive(Debug, thiserror::Error)]
        pub enum ApiError {
            #[error("HTTP error: {0}")]
            Http(#[from] reqwest::Error),

            #[error("Serialization error: {0}")]
            Serialization(#[from] serde_json::Error),

            #[error("API error {status}: {message}")]
            Api { status: u16, message: String },
        }

        pub type ApiResult<T> = Result<T, ApiError>;
    }
}

fn generate_structs(spec: &OpenAPI) -> Result<TokenStream2, String> {
    let mut generated_structs = TokenStream2::new();

    if let Some(components) = &spec.components {
        for (name, schema_ref) in &components.schemas {
            match schema_ref {
                ReferenceOr::Reference { .. } => {
                    // Skip references for now, they should be resolved elsewhere
                    continue;
                }
                ReferenceOr::Item(schema) => {
                    let struct_tokens = generate_struct_from_schema(name, schema)?;
                    generated_structs.extend(struct_tokens);
                }
            }
        }
    }

    Ok(generated_structs)
}

fn generate_struct_from_schema(name: &str, schema: &Schema) -> Result<TokenStream2, String> {
    let struct_name = format_ident!("{}", name.to_pascal_case());
    let doc_comment = generate_doc_comment(schema.schema_data.description.as_deref());

    match &schema.schema_kind {
        SchemaKind::Type(Type::Object(obj)) => {
            let fields = generate_struct_fields_from_object(obj, &schema.schema_data)?;
            Ok(quote! {
                #doc_comment
                #[derive(Debug, Clone, Serialize, Deserialize)]
                pub struct #struct_name {
                    #fields
                }
            })
        }
        SchemaKind::Type(Type::String(string_schema)) if !string_schema.enumeration.is_empty() => {
            let variants = generate_enum_variants_from_string(string_schema)?;
            Ok(quote! {
                #doc_comment
                #[derive(Debug, Clone, Serialize, Deserialize)]
                pub enum #struct_name {
                    #variants
                }
            })
        }
        _ => {
            // For other types, create a type alias
            let rust_type = schema_to_rust_type(schema)?;
            Ok(quote! {
                #doc_comment
                pub type #struct_name = #rust_type;
            })
        }
    }
}

fn generate_struct_fields_from_object(
    obj: &ObjectType,
    _schema_data: &SchemaData,
) -> Result<TokenStream2, String> {
    let mut fields = TokenStream2::new();

    let required_fields: HashSet<String> = obj.required.iter().cloned().collect();

    for (field_name, field_schema_ref) in &obj.properties {
        let snake_case_name = field_name.to_snake_case();
        let field_ident = create_rust_safe_ident(&snake_case_name);
        
        // Generate field documentation and type
        let (field_type, field_doc) = match field_schema_ref {
            ReferenceOr::Reference { reference } => {
                if let Some(type_name) = reference.strip_prefix("#/components/schemas/") {
                    let type_ident = format_ident!("{}", type_name.to_pascal_case());
                    (quote! { #type_ident }, quote! {})
                } else {
                    (quote! { serde_json::Value }, quote! {})
                }
            }
            ReferenceOr::Item(schema) => {
                let rust_type = schema_to_rust_type(schema)?;
                let doc_comment = generate_doc_comment(schema.schema_data.description.as_deref());
                (rust_type, doc_comment)
            }
        };

        let field_type = if required_fields.contains(field_name) {
            field_type
        } else {
            quote! { Option<#field_type> }
        };

        let serde_attr = if field_name != &field_name.to_snake_case() {
            quote! { #[serde(rename = #field_name)] }
        } else {
            quote! {}
        };

        fields.extend(quote! {
            #field_doc
            #serde_attr
            pub #field_ident: #field_type,
        });
    }

    Ok(fields)
}

fn generate_enum_variants_from_string(string_schema: &StringType) -> Result<TokenStream2, String> {
    let mut variants = TokenStream2::new();

    for value in &string_schema.enumeration {
        if let Some(variant_str) = value.as_ref().and_then(|v| Some(v.as_str())) {
            let variant_name = format_ident!("{}", variant_str.to_pascal_case());
            variants.extend(quote! {
                #[serde(rename = #variant_str)]
                #variant_name,
            });
        }
    }

    Ok(variants)
}

fn generate_client_impl(spec: &OpenAPI, client_name: &Ident) -> Result<TokenStream2, String> {
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

    // Build complete impl block
    Ok(quote! {
        impl #client_name {
            /// Create a new API client with the specified base URL
            pub fn new(base_url: impl Into<String>) -> Self {
                Self {
                    base_url: base_url.into(),
                    client: reqwest::Client::new(),
                }
            }

            /// Create a new API client with a custom HTTP client
            pub fn with_client(base_url: impl Into<String>, client: reqwest::Client) -> Self {
                Self {
                    base_url: base_url.into(),
                    client,
                }
            }

            #api_methods
        }
    })
}

fn generate_client_method(
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

    // Generate parameters
    let mut params = TokenStream2::new();
    let mut url_building = quote! {
        let mut url = format!("{}{}", self.base_url, #path);
    };
    let mut query_params = Vec::new();

    for param_ref in &operation.parameters {
        let param = match param_ref {
            ReferenceOr::Reference { reference } => {
                return Err(format!("Parameter references not supported: {}", reference));
            }
            ReferenceOr::Item(item) => item,
        };

        let (param_name, param_schema, param_location) = match param {
            openapiv3::Parameter::Query { parameter_data, .. } => {
                (&parameter_data.name, &parameter_data.format, "query")
            }
            openapiv3::Parameter::Path { parameter_data, .. } => {
                (&parameter_data.name, &parameter_data.format, "path")
            }
            openapiv3::Parameter::Header { parameter_data, .. } => {
                (&parameter_data.name, &parameter_data.format, "header")
            }
            openapiv3::Parameter::Cookie { parameter_data, .. } => {
                (&parameter_data.name, &parameter_data.format, "cookie")
            }
        };

        let snake_case_param = param_name.to_snake_case();
        let param_ident = create_rust_safe_ident(&snake_case_param);

        let param_type = match param_schema {
            openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => {
                reference_or_schema_to_rust_type(schema_ref)?
            }
            _ => quote! { String },
        };

        // Check if this is an array parameter
        let is_array_param = match param_schema {
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

        params.extend(quote! { #param_ident: #param_type, });

        // Handle different parameter types
        match param_location {
            "path" => {
                let placeholder = format!("{{{}}}", param_name);
                url_building = quote! {
                    let mut url = format!("{}{}", self.base_url, #path).replace(#placeholder, &#param_ident.to_string());
                };
            }
            "query" => {
                query_params.push((param_name, param_ident, is_array_param));
            }
            _ => {
                // Header and cookie parameters not implemented yet
            }
        }
    }

    // Add query parameters to URL building using the url crate
    if !query_params.is_empty() {
        let query_building = query_params.iter().map(|(param_name, param_ident, is_array)| {
            if *is_array {
                quote! {
                    // Handle array parameters by joining with commas
                    let param_value = #param_ident.join(",");
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

    // Determine return type
    let return_type =
        determine_return_type_from_operation(operation).unwrap_or_else(|| quote! { () });

    // Generate documentation from operation summary and description
    let doc_comment = generate_method_doc_comment(operation, path, http_method);

    Ok(quote! {
        #doc_comment
        pub async fn #method_name(&self, #params #body_param) -> ApiResult<#return_type> {
            #url_building
            #request_building

            let response = request.send().await?;

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
    })
}

fn determine_return_type_from_operation(operation: &openapiv3::Operation) -> Option<TokenStream2> {
    let response_200 = operation
        .responses
        .responses
        .get(&openapiv3::StatusCode::Code(200))?;
    let response = match response_200 {
        ReferenceOr::Reference { .. } => return None,
        ReferenceOr::Item(item) => item,
    };
    let content = response.content.get("application/json")?;
    let schema_ref = content.schema.as_ref()?;
    reference_or_schema_to_rust_type(schema_ref).ok()
}

fn reference_or_schema_to_rust_type(
    schema_ref: &ReferenceOr<Schema>,
) -> Result<TokenStream2, String> {
    match schema_ref {
        ReferenceOr::Reference { reference } => {
            if let Some(type_name) = reference.strip_prefix("#/components/schemas/") {
                let type_ident = format_ident!("{}", type_name.to_pascal_case());
                Ok(quote! { #type_ident })
            } else {
                Ok(quote! { serde_json::Value })
            }
        }
        ReferenceOr::Item(schema) => schema_to_rust_type(schema),
    }
}

fn schema_to_rust_type(schema: &Schema) -> Result<TokenStream2, String> {
    match &schema.schema_kind {
        SchemaKind::Type(Type::String(_)) => Ok(quote! { String }),
        SchemaKind::Type(Type::Integer(int_schema)) => match int_schema.format {
            openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::IntegerFormat::Int64) => {
                Ok(quote! { i64 })
            }
            _ => Ok(quote! { i32 }),
        },
        SchemaKind::Type(Type::Number(num_schema)) => match num_schema.format {
            openapiv3::VariantOrUnknownOrEmpty::Item(openapiv3::NumberFormat::Double) => {
                Ok(quote! { f64 })
            }
            _ => Ok(quote! { f32 }),
        },
        SchemaKind::Type(Type::Boolean(_)) => Ok(quote! { bool }),
        SchemaKind::Type(Type::Array(array_schema)) => {
            if let Some(items) = &array_schema.items {
                let item_type = match items {
                    ReferenceOr::Reference { reference } => {
                        if let Some(type_name) = reference.strip_prefix("#/components/schemas/") {
                            let type_ident = format_ident!("{}", type_name.to_pascal_case());
                            quote! { #type_ident }
                        } else {
                            quote! { serde_json::Value }
                        }
                    }
                    ReferenceOr::Item(schema) => schema_to_rust_type(schema)?,
                };
                Ok(quote! { Vec<#item_type> })
            } else {
                Ok(quote! { Vec<serde_json::Value> })
            }
        }
        SchemaKind::Type(Type::Object(_)) => Ok(quote! { HashMap<String, serde_json::Value> }),
        _ => Ok(quote! { serde_json::Value }),
    }
}

/// Generate documentation comment from description text
fn generate_doc_comment(description: Option<&str>) -> TokenStream2 {
    if let Some(desc) = description {
        if !desc.trim().is_empty() {
            let clean_desc = desc
                .lines()
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            
            return quote! {
                #[doc = #clean_desc]
            };
        }
    }
    quote! {}
}

/// Generate documentation comment for the API client
fn generate_client_doc_comment(spec: &OpenAPI, client_name: &str) -> TokenStream2 {
    let mut doc_lines = Vec::new();
    
    // Add API title as the first line
    if !spec.info.title.trim().is_empty() {
        doc_lines.push(format!("API Client for {}", spec.info.title.trim()));
    } else {
        doc_lines.push(format!("Generated API Client: {}", client_name));
    }
    
    // Add API description if available
    if let Some(description) = &spec.info.description {
        let clean_desc = description.trim();
        if !clean_desc.is_empty() {
            doc_lines.push("".to_string()); // Empty line separator
            doc_lines.push(clean_desc.to_string());
        }
    }
    
    // Add API version
    if !spec.info.version.trim().is_empty() {
        doc_lines.push("".to_string()); // Empty line separator
        doc_lines.push(format!("**API Version:** `{}`", spec.info.version.trim()));
    }
    
    // Add contact information if available
    if let Some(contact) = &spec.info.contact {
        if let Some(email) = &contact.email {
            doc_lines.push(format!("**Contact:** {}", email));
        }
    }
    
    // Add license information if available
    if let Some(license) = &spec.info.license {
        if !license.name.trim().is_empty() {
            let license_info = if let Some(url) = &license.url {
                format!("**License:** [{}]({})", license.name, url)
            } else {
                format!("**License:** {}", license.name)
            };
            doc_lines.push(license_info);
        }
    }
    
    // Add terms of service if available
    if let Some(terms) = &spec.info.terms_of_service {
        if !terms.trim().is_empty() {
            doc_lines.push(format!("**Terms of Service:** {}", terms));
        }
    }
    
    // Add usage example
    doc_lines.push("".to_string()); // Empty line separator
    doc_lines.push("# Example".to_string());
    doc_lines.push("```rust".to_string());
    doc_lines.push(format!("let client = {}::new(\"https://api.example.com\");", client_name));
    doc_lines.push("let result = client.some_method().await?;".to_string());
    doc_lines.push("```".to_string());
    
    // Generate doc attributes for each line
    let doc_attrs = doc_lines.iter().map(|line| {
        quote! { #[doc = #line] }
    });
    
    quote! {
        #(#doc_attrs)*
    }
}

/// Generate documentation comment for API methods
fn generate_method_doc_comment(
    operation: &openapiv3::Operation,
    path: &str,
    http_method: &str,
) -> TokenStream2 {
    let mut doc_lines = Vec::new();
    
    // Add summary as the first line
    if let Some(summary) = &operation.summary {
        if !summary.trim().is_empty() {
            doc_lines.push(summary.trim().to_string());
        }
    }
    
    // Add description if available and different from summary
    if let Some(description) = &operation.description {
        let clean_desc = description.trim();
        if !clean_desc.is_empty() && Some(clean_desc) != operation.summary.as_deref() {
            if !doc_lines.is_empty() {
                doc_lines.push("".to_string()); // Empty line separator
            }
            doc_lines.push(clean_desc.to_string());
        }
    }
    
    // Add HTTP method and path info
    if !doc_lines.is_empty() {
        doc_lines.push("".to_string()); // Empty line separator
    }
    doc_lines.push(format!("**HTTP Method:** `{}`", http_method.to_uppercase()));
    doc_lines.push(format!("**Path:** `{}`", path));
    
    // Add operation ID if available
    if let Some(operation_id) = &operation.operation_id {
        doc_lines.push(format!("**Operation ID:** `{}`", operation_id));
    }
    
    if doc_lines.is_empty() {
        return quote! {};
    }
    
    // Generate doc attributes for each line
    let doc_attrs = doc_lines.iter().map(|line| {
        quote! { #[doc = #line] }
    });
    
    quote! {
        #(#doc_attrs)*
    }
}

fn is_rust_keyword(name: &str) -> bool {
    matches!(
        name,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
    )
}

/// Create a Rust-safe identifier, prefixing with r# if it's a keyword
fn create_rust_safe_ident(name: &str) -> Ident {
    if is_rust_keyword(name) {
        // Special handling for keywords that cannot be raw identifiers
        match name {
            "self" => format_ident!("self_"),
            "Self" => format_ident!("Self_"),
            _ => format_ident!("r#{}", name),
        }
    } else {
        format_ident!("{}", name)
    }
}

/// Check if a path is a URL (starts with http:// or https://)
fn is_url(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://")
}

/// Check if a path indicates YAML format (file extension or URL path)
fn is_yaml_format(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    path_lower.ends_with(".yaml") || path_lower.ends_with(".yml")
}

/// Fetch content from a URL at compile time
fn fetch_url_content(url: &str) -> Result<String, String> {
    // Use blocking reqwest for compile-time execution
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;
    
    rt.block_on(async {
        let client = reqwest::Client::new();
        let response = client.get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch URL {}: {}", url, e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error {} when fetching {}", response.status(), url));
        }

        response.text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))
    })
}
