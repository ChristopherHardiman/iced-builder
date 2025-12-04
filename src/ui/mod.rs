//! UI components for the Iced Builder application.
//!
//! This module contains the visual components of the builder:
//! - Widget palette (left sidebar)
//! - Canvas/viewport (center)
//! - Property inspector (right sidebar)
//! - Tree view (optional bottom/left panel)

pub mod canvas;
pub mod inspector;
pub mod palette;
pub mod tree_view;

pub use canvas::Canvas;
pub use inspector::Inspector;
pub use palette::Palette;
pub use tree_view::TreeView;
