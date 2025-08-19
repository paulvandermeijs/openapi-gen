//! Code generation components for OpenAPI client generation.
//!
//! This module contains the core code generation logic that transforms
//! parsed OpenAPI specifications into Rust client code.

pub mod client;
pub mod docs;
pub mod errors;
pub mod methods;
pub mod param_structs;
pub mod structs;

pub use client::*;
pub use docs::*;
pub use errors::*;
pub use param_structs::*;
pub use structs::*;
