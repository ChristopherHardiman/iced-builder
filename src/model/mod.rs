//! Data model types for the Iced Builder application.
//!
//! This module contains the Layout AST, project configuration,
//! and undo/redo history management.

pub mod history;
pub mod layout;
pub mod project;

pub use history::History;
pub use layout::{ComponentId, LayoutDocument, LayoutNode};
pub use project::{Project, ProjectConfig};
