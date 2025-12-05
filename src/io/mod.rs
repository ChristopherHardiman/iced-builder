//! File I/O module.
//!
//! Handles loading and saving layout files and project configuration.

pub mod config;
pub mod layout_file;

// Re-exports for convenience
#[allow(unused_imports)]
pub use config::{
    config_path, find_config, is_valid_project, load_config, save_config, ConfigError,
    CONFIG_FILENAME,
};
#[allow(unused_imports)]
pub use layout_file::{
    default_layout_path, find_layout_files, load_layout, save_layout, LayoutFileError,
    LayoutFormat,
};
