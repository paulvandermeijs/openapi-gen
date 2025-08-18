use proc_macro2::Ident;
use quote::format_ident;

/// Check if a string is a Rust keyword
pub fn is_rust_keyword(name: &str) -> bool {
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
pub fn create_rust_safe_ident(name: &str) -> Ident {
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
