//! Code generation module.
//!
//! Converts the Layout AST to Rust/Iced source code.

pub mod generator;

pub use generator::generate_code;
