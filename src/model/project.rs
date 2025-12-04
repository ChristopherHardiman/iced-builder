//! Project configuration and state management.
//!
//! Handles loading/saving project configuration from `iced_builder.toml`
//! and managing the overall project state.

use crate::model::{layout::NodeIndex, ComponentId, History, LayoutDocument, LayoutNode};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when working with projects.
#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("Failed to read config file: {0}")]
    ConfigRead(#[from] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("Config file not found: {0}")]
    ConfigNotFound(PathBuf),

    #[error("Layout file not found: {0}")]
    LayoutNotFound(PathBuf),

    #[error("Failed to parse layout file: {0}")]
    LayoutParse(String),
}

/// Project configuration loaded from `iced_builder.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Path to the target project root (optional, defaults to config location).
    #[serde(default)]
    pub project_root: Option<PathBuf>,

    /// Relative path for generated Rust code output.
    #[serde(default = "default_output_file")]
    pub output_file: PathBuf,

    /// Fully-qualified Rust type for messages (e.g., `crate::Message`).
    #[serde(default = "default_message_type")]
    pub message_type: String,

    /// Fully-qualified Rust type for app state (e.g., `crate::AppState`).
    #[serde(default = "default_state_type")]
    pub state_type: String,

    /// List or glob of layout files to load.
    #[serde(default)]
    pub layout_files: Vec<PathBuf>,

    /// Whether to run rustfmt on generated code.
    #[serde(default = "default_true")]
    pub format_output: bool,
}

fn default_output_file() -> PathBuf {
    PathBuf::from("src/ui/layout_generated.rs")
}

fn default_message_type() -> String {
    String::from("crate::Message")
}

fn default_state_type() -> String {
    String::from("crate::AppState")
}

fn default_true() -> bool {
    true
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project_root: None,
            output_file: default_output_file(),
            message_type: default_message_type(),
            state_type: default_state_type(),
            layout_files: Vec::new(),
            format_output: true,
        }
    }
}

impl ProjectConfig {
    /// Load project configuration from a TOML file.
    pub fn load(path: &std::path::Path) -> Result<Self, ProjectError> {
        if !path.exists() {
            return Err(ProjectError::ConfigNotFound(path.to_path_buf()));
        }
        let content = std::fs::read_to_string(path)?;
        let config: ProjectConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save project configuration to a TOML file.
    pub fn save(&self, path: &std::path::Path) -> Result<(), ProjectError> {
        let content =
            toml::to_string_pretty(self).map_err(|e| ProjectError::LayoutParse(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// The complete state of an open project.
#[derive(Debug, Clone)]
pub struct Project {
    /// Path to the project directory.
    pub path: PathBuf,

    /// Project configuration.
    pub config: ProjectConfig,

    /// The current layout document.
    pub layout: LayoutDocument,

    /// Index for O(1) node lookup by ID.
    pub node_index: NodeIndex,

    /// Currently selected component.
    pub selected_id: Option<ComponentId>,

    /// Undo/redo history.
    pub history: History,

    /// Whether there are unsaved changes.
    pub dirty: bool,
}

impl Project {
    /// Create a new project with default layout.
    pub fn new(path: PathBuf, config: ProjectConfig) -> Self {
        let layout = LayoutDocument::default();
        let node_index = crate::model::layout::build_node_index(&layout.root);

        Self {
            path,
            config,
            layout,
            node_index,
            selected_id: None,
            history: History::new(),
            dirty: false,
        }
    }

    /// Rebuild the node index after structural changes.
    pub fn rebuild_index(&mut self) {
        self.node_index = crate::model::layout::build_node_index(&self.layout.root);
    }

    /// Find a node by its ComponentId.
    pub fn find_node(&self, id: ComponentId) -> Option<&LayoutNode> {
        let path = self.node_index.get(&id)?;
        self.find_node_by_path(&self.layout.root, path)
    }

    /// Find a node by path (helper).
    fn find_node_by_path<'a>(&self, root: &'a LayoutNode, path: &[usize]) -> Option<&'a LayoutNode> {
        if path.is_empty() {
            return Some(root);
        }

        let idx = path[0];
        let remaining = &path[1..];

        if let Some(children) = root.children() {
            if idx < children.len() {
                return self.find_node_by_path(&children[idx], remaining);
            }
        }

        // Handle single-child containers
        match &root.widget {
            crate::model::layout::WidgetType::Container { child: Some(c), .. }
            | crate::model::layout::WidgetType::Scrollable { child: Some(c), .. } => {
                if idx == 0 {
                    return self.find_node_by_path(c, remaining);
                }
            }
            _ => {}
        }

        None
    }

    /// Mark the project as having unsaved changes.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Mark the project as saved.
    pub fn mark_saved(&mut self) {
        self.dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_config_default() {
        let config = ProjectConfig::default();
        assert_eq!(config.message_type, "crate::Message");
        assert_eq!(config.state_type, "crate::AppState");
        assert!(config.format_output);
    }

    #[test]
    fn test_project_new() {
        let config = ProjectConfig::default();
        let project = Project::new(PathBuf::from("/test"), config);
        assert!(project.selected_id.is_none());
        assert!(!project.dirty);
    }
}
