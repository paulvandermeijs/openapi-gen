use openapiv3::{ReferenceOr, Schema, SchemaKind, Type};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use heck::ToPascalCase;

/// Convert an OpenAPI schema to a Rust type
pub fn schema_to_rust_type(schema: &Schema) -> Result<TokenStream2, String> {
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

/// Convert a reference or schema to a Rust type
pub fn reference_or_schema_to_rust_type(
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