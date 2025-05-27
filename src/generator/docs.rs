use openapiv3::{OpenAPI, Operation};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generate documentation comment from description text
pub fn generate_doc_comment(description: Option<&str>) -> TokenStream2 {
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
pub fn generate_client_doc_comment(spec: &OpenAPI, client_name: &str) -> TokenStream2 {
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
pub fn generate_method_doc_comment(
    operation: &Operation,
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