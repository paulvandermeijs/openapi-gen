//! OpenAPI specification parsing and loading.
//!
//! This module handles loading OpenAPI specifications from files or URLs
//! and parsing macro input arguments.

pub mod input;
pub mod loader;
pub mod spec;

pub use input::*;
pub use loader::*;
pub use spec::*;
