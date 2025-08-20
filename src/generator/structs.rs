use heck::{ToPascalCase, ToSnakeCase};
use openapiv3::{
    ObjectType, OpenAPI, ReferenceOr, Schema, SchemaData, SchemaKind, StringType, Type,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::collections::HashSet;

use crate::codegen::schema_to_rust_type;
use crate::generator::docs::generate_doc_comment;
use crate::utils::create_rust_safe_ident;

/// Generate all structs from OpenAPI components
pub fn generate_structs(
    spec: &OpenAPI,
    struct_attrs: &[TokenStream2],
) -> Result<TokenStream2, String> {
    let mut generated_structs = TokenStream2::new();

    if let Some(components) = &spec.components {
        for (name, schema_ref) in &components.schemas {
            match schema_ref {
                ReferenceOr::Reference { .. } => {
                    // Skip references for now, they should be resolved elsewhere
                    continue;
                }
                ReferenceOr::Item(schema) => {
                    let struct_tokens = generate_struct_from_schema(name, schema, struct_attrs)?;
                    generated_structs.extend(struct_tokens);
                }
            }
        }
    }

    Ok(generated_structs)
}

/// Generate a struct from an OpenAPI schema
fn generate_struct_from_schema(
    name: &str,
    schema: &Schema,
    struct_attrs: &[TokenStream2],
) -> Result<TokenStream2, String> {
    let struct_name = format_ident!("{}", name.to_pascal_case());
    let doc_comment = generate_doc_comment(schema.schema_data.description.as_deref());

    match &schema.schema_kind {
        SchemaKind::Type(Type::Object(obj)) => {
            let fields = generate_struct_fields_from_object(name, obj, &schema.schema_data)?;

            // Convert user attribute token streams to attributes
            let user_attrs = struct_attrs.iter().map(|tokens| {
                quote! { #[#tokens] }
            });

            Ok(quote! {
                #doc_comment
                #(#user_attrs)*
                #[derive(Debug, Clone, Serialize, Deserialize)]
                pub struct #struct_name {
                    #fields
                }
            })
        }
        SchemaKind::Type(Type::String(string_schema)) if !string_schema.enumeration.is_empty() => {
            let variants = generate_enum_variants_from_string(string_schema)?;

            // Convert user attribute token streams to attributes
            let user_attrs = struct_attrs.iter().map(|tokens| {
                quote! { #[#tokens] }
            });

            Ok(quote! {
                #doc_comment
                #(#user_attrs)*
                #[derive(Debug, Clone, Serialize, Deserialize)]
                pub enum #struct_name {
                    #variants
                }
            })
        }
        _ => {
            // For other types, create a type alias (attributes don't apply to type aliases)
            let rust_type = schema_to_rust_type(schema)?;
            Ok(quote! {
                #doc_comment
                pub type #struct_name = #rust_type;
            })
        }
    }
}

/// Generate struct fields from an object type
fn generate_struct_fields_from_object(
    struct_name: &str,
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
                    let ty = if type_name == struct_name {
                        quote! { Box<#type_ident> }
                    } else {
                        quote! { #type_ident }
                    };
                    (ty, quote! {})
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

/// Generate enum variants from a string schema
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
