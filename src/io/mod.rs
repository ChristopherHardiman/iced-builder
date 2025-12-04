//! File I/O module.
//!
//! Handles loading and saving layout files and project configuration.

pub mod config;
pub mod layout_file;

// Re-exports (unused in Phase 0 but available for future phases)
#[allow(unused_imports)]
pub use config::load_config;
#[allow(unused_imports)]
pub use layout_file::{load_layout, save_layout};
