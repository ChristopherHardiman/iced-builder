//! Debug logging system for Iced Builder.
//!
//! Provides structured logging with configurable levels via environment variables.
//! 
//! # Usage
//! 
//! Set the `ICED_BUILDER_LOG` environment variable to control log levels:
//! 
//! ```bash
//! # Show all debug messages
//! ICED_BUILDER_LOG=debug cargo run
//! 
//! # Show info and above
//! ICED_BUILDER_LOG=info cargo run
//! 
//! # Show only warnings and errors
//! ICED_BUILDER_LOG=warn cargo run
//! 
//! # Fine-grained control per module
//! ICED_BUILDER_LOG=iced_builder::app=debug,iced_builder::codegen=trace cargo run
//! ```
//!
//! # Log Levels
//! 
//! - `error` - Critical failures that prevent operation
//! - `warn` - Potential issues or unexpected conditions
//! - `info` - High-level application events (startup, file operations)
//! - `debug` - Detailed operational information (message handling, state changes)
//! - `trace` - Very detailed debugging (widget rendering, AST traversal)

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize the logging system.
/// 
/// Call this at the start of `main()` before any other operations.
/// 
/// Reads log level from `ICED_BUILDER_LOG` environment variable.
/// Defaults to `info` if not set.
pub fn init() {
    let filter = EnvFilter::try_from_env("ICED_BUILDER_LOG")
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(fmt::layer()
            .with_target(true)
            .with_thread_ids(false)
            .with_file(true)
            .with_line_number(true)
            .compact())
        .with(filter)
        .init();

    tracing::info!("Iced Builder logging initialized");
}

/// Log categories for different subsystems.
/// 
/// These are used as targets for filtering log output.
pub mod targets {
    /// Application-level events (startup, shutdown, mode changes)
    pub const APP: &str = "iced_builder::app";
    
    /// Message handling and state updates
    pub const MESSAGE: &str = "iced_builder::app::message";
    
    /// Selection and interaction events
    pub const SELECTION: &str = "iced_builder::app::selection";
    
    /// Widget tree modifications
    pub const TREE: &str = "iced_builder::app::tree";
    
    /// Code generation events
    pub const CODEGEN: &str = "iced_builder::codegen";
    
    /// File I/O operations
    pub const IO: &str = "iced_builder::io";
    
    /// Canvas rendering
    pub const CANVAS: &str = "iced_builder::ui::canvas";
    
    /// Inspector property changes
    pub const INSPECTOR: &str = "iced_builder::ui::inspector";
    
    /// Palette interactions
    pub const PALETTE: &str = "iced_builder::ui::palette";
}

/// Convenience macros for logging with predefined targets.
/// 
/// These wrap the tracing macros with the appropriate target.
#[macro_export]
macro_rules! log_message {
    ($($arg:tt)*) => {
        tracing::debug!(target: $crate::logging::targets::MESSAGE, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_selection {
    ($($arg:tt)*) => {
        tracing::debug!(target: $crate::logging::targets::SELECTION, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_tree {
    ($($arg:tt)*) => {
        tracing::debug!(target: $crate::logging::targets::TREE, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_codegen {
    ($($arg:tt)*) => {
        tracing::debug!(target: $crate::logging::targets::CODEGEN, $($arg)*)
    };
}
