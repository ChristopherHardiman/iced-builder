//! Project configuration and state management.
//!
//! Handles loading/saving project configuration from `iced_builder.toml`
//! and managing the overall project state.

use crate::io::{config, layout_file};
use crate::model::{layout::NodeIndex, ComponentId, History, LayoutDocument, LayoutNode};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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

    /// Open an existing project from a directory.
    ///
    /// Looks for `iced_builder.toml` in the given directory, loads configuration,
    /// then loads the layout file(s) specified in the config.
    pub fn open(project_dir: &Path) -> Result<Self, ProjectError> {
        tracing::info!(target: "iced_builder::io", path = %project_dir.display(), "Opening project");

        // Find and load config file
        let config_path = config::find_config(project_dir)
            .ok_or_else(|| ProjectError::ConfigNotFound(project_dir.join("iced_builder.toml")))?;
        
        let config = config::load_config(&config_path)
            .map_err(|e| match e {
                config::ConfigError::ReadError(io) => ProjectError::ConfigRead(io),
                config::ConfigError::ParseError(p) => ProjectError::ConfigParse(p),
                config::ConfigError::NotFound(s) => ProjectError::ConfigNotFound(PathBuf::from(s)),
                config::ConfigError::SerializeError(_) => {
                    ProjectError::LayoutParse("Config serialize error".to_string())
                }
            })?;

        tracing::debug!(target: "iced_builder::io", ?config, "Config loaded");

        // Load layout file
        let layout = Self::load_layout_for_project(project_dir, &config)?;
        let node_index = crate::model::layout::build_node_index(&layout.root);

        tracing::info!(
            target: "iced_builder::io", 
            name = %layout.name, 
            node_count = node_index.len(),
            "Project opened successfully"
        );

        Ok(Self {
            path: project_dir.to_path_buf(),
            config,
            layout,
            node_index,
            selected_id: None,
            history: History::new(),
            dirty: false,
        })
    }

    /// Load the layout file for a project.
    fn load_layout_for_project(project_dir: &Path, config: &ProjectConfig) -> Result<LayoutDocument, ProjectError> {
        // Try layout files from config first
        if !config.layout_files.is_empty() {
            for layout_path in &config.layout_files {
                let full_path = project_dir.join(layout_path);
                if full_path.exists() {
                    tracing::debug!(target: "iced_builder::io", path = %full_path.display(), "Loading layout from config");
                    return layout_file::load_layout(&full_path)
                        .map_err(|e| ProjectError::LayoutParse(e.to_string()));
                }
            }
        }

        // Fall back to default layout.ron
        let default_path = project_dir.join("layout.ron");
        if default_path.exists() {
            tracing::debug!(target: "iced_builder::io", path = %default_path.display(), "Loading default layout.ron");
            return layout_file::load_layout(&default_path)
                .map_err(|e| ProjectError::LayoutParse(e.to_string()));
        }

        // Try layout.json as alternative
        let json_path = project_dir.join("layout.json");
        if json_path.exists() {
            tracing::debug!(target: "iced_builder::io", path = %json_path.display(), "Loading layout.json");
            return layout_file::load_layout(&json_path)
                .map_err(|e| ProjectError::LayoutParse(e.to_string()));
        }

        // No layout found - return error
        Err(ProjectError::LayoutNotFound(default_path))
    }

    /// Save the project to disk.
    ///
    /// Saves both the configuration and the layout file.
    pub fn save(&mut self) -> Result<(), ProjectError> {
        tracing::info!(target: "iced_builder::io", path = %self.path.display(), "Saving project");

        // Save config
        let config_path = self.path.join("iced_builder.toml");
        config::save_config(&config_path, &self.config)
            .map_err(|e| match e {
                config::ConfigError::ReadError(io) => ProjectError::ConfigRead(io),
                config::ConfigError::SerializeError(s) => ProjectError::LayoutParse(s.to_string()),
                _ => ProjectError::LayoutParse("Config save error".to_string()),
            })?;

        // Determine layout file path
        let layout_path = if !self.config.layout_files.is_empty() {
            self.path.join(&self.config.layout_files[0])
        } else {
            self.path.join("layout.ron")
        };

        // Save layout
        layout_file::save_layout(&layout_path, &self.layout)
            .map_err(|e| ProjectError::LayoutParse(e.to_string()))?;

        self.dirty = false;
        tracing::info!(target: "iced_builder::io", "Project saved successfully");
        Ok(())
    }

    /// Export generated Rust code to the configured output file.
    pub fn export(&self) -> Result<String, ProjectError> {
        tracing::info!(target: "iced_builder::codegen", "Exporting code");

        let code = crate::codegen::generate_code(&self.layout, &self.config);
        let formatted = if self.config.format_output {
            crate::util::try_format_rust_code(&code)
        } else {
            code
        };

        // Determine output path
        let output_path = if self.config.output_file.is_absolute() {
            self.config.output_file.clone()
        } else {
            self.path.join(&self.config.output_file)
        };

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create backup if file exists
        if output_path.exists() {
            let backup_path = output_path.with_extension("rs.bak");
            let _ = std::fs::copy(&output_path, backup_path);
        }

        // Write the generated code
        std::fs::write(&output_path, &formatted)?;

        tracing::info!(
            target: "iced_builder::codegen", 
            path = %output_path.display(), 
            size = formatted.len(),
            "Code exported successfully"
        );

        Ok(formatted)
    }

    /// Create a new project in the given directory.
    ///
    /// Creates the config file and an initial layout file.
    pub fn create(project_dir: &Path, template: Option<Template>) -> Result<Self, ProjectError> {
        tracing::info!(target: "iced_builder::io", path = %project_dir.display(), "Creating new project");

        // Ensure directory exists
        std::fs::create_dir_all(project_dir)?;

        // Create config file
        let config = ProjectConfig::default();
        let config_path = project_dir.join("iced_builder.toml");
        config::save_config(&config_path, &config)
            .map_err(|e| match e {
                config::ConfigError::ReadError(io) => ProjectError::ConfigRead(io),
                config::ConfigError::SerializeError(s) => ProjectError::LayoutParse(s.to_string()),
                _ => ProjectError::LayoutParse("Config create error".to_string()),
            })?;

        // Create layout file from template or default
        let layout = match template {
            Some(Template::Form) => Self::create_form_template(),
            Some(Template::Dashboard) => Self::create_dashboard_template(),
            None | Some(Template::Blank) => LayoutDocument::default(),
        };

        let layout_path = project_dir.join("layout.ron");
        layout_file::save_layout(&layout_path, &layout)
            .map_err(|e| ProjectError::LayoutParse(e.to_string()))?;

        let node_index = crate::model::layout::build_node_index(&layout.root);

        tracing::info!(target: "iced_builder::io", "New project created successfully");

        Ok(Self {
            path: project_dir.to_path_buf(),
            config,
            layout,
            node_index,
            selected_id: None,
            history: History::new(),
            dirty: false,
        })
    }

    /// Create a form template layout.
    fn create_form_template() -> LayoutDocument {
        use crate::model::layout::*;
        
        LayoutDocument {
            version: 1,
            name: String::from("Form"),
            root: LayoutNode::new(WidgetType::Column {
                children: vec![
                    LayoutNode::new(WidgetType::Text {
                        content: String::from("Form Title"),
                        attrs: TextAttrs {
                            font_size: 24.0,
                            ..Default::default()
                        },
                    }),
                    LayoutNode::new(WidgetType::TextInput {
                        placeholder: String::from("Enter your name..."),
                        value_binding: String::from("name"),
                        message_stub: String::from("NameChanged"),
                        attrs: InputAttrs::default(),
                    }),
                    LayoutNode::new(WidgetType::TextInput {
                        placeholder: String::from("Enter your email..."),
                        value_binding: String::from("email"),
                        message_stub: String::from("EmailChanged"),
                        attrs: InputAttrs::default(),
                    }),
                    LayoutNode::new(WidgetType::Button {
                        label: String::from("Submit"),
                        message_stub: String::from("Submit"),
                        attrs: ButtonAttrs::default(),
                    }),
                ],
                attrs: ContainerAttrs {
                    spacing: 10.0,
                    padding: PaddingSpec { top: 20.0, right: 20.0, bottom: 20.0, left: 20.0 },
                    ..Default::default()
                },
            }),
        }
    }

    /// Create a dashboard template layout.
    fn create_dashboard_template() -> LayoutDocument {
        use crate::model::layout::*;
        
        LayoutDocument {
            version: 1,
            name: String::from("Dashboard"),
            root: LayoutNode::new(WidgetType::Column {
                children: vec![
                    // Header row
                    LayoutNode::new(WidgetType::Row {
                        children: vec![
                            LayoutNode::new(WidgetType::Text {
                                content: String::from("Dashboard"),
                                attrs: TextAttrs {
                                    font_size: 28.0,
                                    ..Default::default()
                                },
                            }),
                            LayoutNode::new(WidgetType::Space {
                                width: LengthSpec::Fill,
                                height: LengthSpec::Shrink,
                            }),
                            LayoutNode::new(WidgetType::Button {
                                label: String::from("Settings"),
                                message_stub: String::from("OpenSettings"),
                                attrs: ButtonAttrs::default(),
                            }),
                        ],
                        attrs: ContainerAttrs {
                            spacing: 10.0,
                            ..Default::default()
                        },
                    }),
                    // Content row
                    LayoutNode::new(WidgetType::Row {
                        children: vec![
                            // Left panel
                            LayoutNode::new(WidgetType::Column {
                                children: vec![
                                    LayoutNode::new(WidgetType::Text {
                                        content: String::from("Statistics"),
                                        attrs: TextAttrs::default(),
                                    }),
                                ],
                                attrs: ContainerAttrs {
                                    width: LengthSpec::FillPortion(1),
                                    ..Default::default()
                                },
                            }),
                            // Right panel
                            LayoutNode::new(WidgetType::Column {
                                children: vec![
                                    LayoutNode::new(WidgetType::Text {
                                        content: String::from("Activity"),
                                        attrs: TextAttrs::default(),
                                    }),
                                ],
                                attrs: ContainerAttrs {
                                    width: LengthSpec::FillPortion(2),
                                    ..Default::default()
                                },
                            }),
                        ],
                        attrs: ContainerAttrs {
                            spacing: 20.0,
                            height: LengthSpec::Fill,
                            ..Default::default()
                        },
                    }),
                ],
                attrs: ContainerAttrs {
                    spacing: 20.0,
                    padding: PaddingSpec { top: 20.0, right: 20.0, bottom: 20.0, left: 20.0 },
                    height: LengthSpec::Fill,
                    width: LengthSpec::Fill,
                    ..Default::default()
                },
            }),
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

    /// Find a mutable node by its ComponentId.
    pub fn find_node_mut(&mut self, id: ComponentId) -> Option<&mut LayoutNode> {
        let path = self.node_index.get(&id)?.clone();
        Self::find_node_by_path_mut_static(&mut self.layout.root, &path)
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

    /// Find a mutable node by path (static helper to avoid borrow issues).
    fn find_node_by_path_mut_static<'a>(root: &'a mut LayoutNode, path: &[usize]) -> Option<&'a mut LayoutNode> {
        if path.is_empty() {
            return Some(root);
        }

        let idx = path[0];
        let remaining = &path[1..];

        // Check if this is a multi-child container
        match &mut root.widget {
            crate::model::layout::WidgetType::Column { children, .. }
            | crate::model::layout::WidgetType::Row { children, .. }
            | crate::model::layout::WidgetType::Stack { children, .. } => {
                if idx < children.len() {
                    return Self::find_node_by_path_mut_static(&mut children[idx], remaining);
                }
            }
            crate::model::layout::WidgetType::Container { child: Some(c), .. }
            | crate::model::layout::WidgetType::Scrollable { child: Some(c), .. } => {
                if idx == 0 {
                    return Self::find_node_by_path_mut_static(c, remaining);
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

    /// Get the project directory path.
    pub fn project_path(&self) -> &Path {
        &self.path
    }

    /// Remove a node from the tree by its ComponentId.
    /// Returns true if the node was found and removed.
    /// Note: The root node cannot be removed.
    pub fn remove_node(&mut self, id: ComponentId) -> bool {
        // Get the path to the node
        let path = match self.node_index.get(&id) {
            Some(p) => p.clone(),
            None => return false,
        };

        // Cannot remove root node (empty path)
        if path.is_empty() {
            return false;
        }

        // Find the parent and remove the child
        let parent_path = &path[..path.len() - 1];
        let child_index = path[path.len() - 1];

        let removed = if parent_path.is_empty() {
            // Parent is root
            Self::remove_child_at(&mut self.layout.root, child_index)
        } else {
            // Find parent node
            if let Some(parent) = Self::find_node_by_path_mut_static(&mut self.layout.root, parent_path) {
                Self::remove_child_at(parent, child_index)
            } else {
                false
            }
        };

        if removed {
            self.rebuild_index();
        }

        removed
    }

    /// Remove a child at a specific index from a node.
    fn remove_child_at(node: &mut LayoutNode, index: usize) -> bool {
        match &mut node.widget {
            crate::model::layout::WidgetType::Column { children, .. }
            | crate::model::layout::WidgetType::Row { children, .. }
            | crate::model::layout::WidgetType::Stack { children, .. } => {
                if index < children.len() {
                    children.remove(index);
                    return true;
                }
            }
            crate::model::layout::WidgetType::Container { child, .. }
            | crate::model::layout::WidgetType::Scrollable { child, .. } => {
                if index == 0 && child.is_some() {
                    *child = None;
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    /// Add a child node to a container by ComponentId.
    /// Returns true if the child was successfully added.
    /// Returns false if the target node is not a container or doesn't exist.
    pub fn add_child_to_node(&mut self, parent_id: ComponentId, new_child: LayoutNode) -> bool {
        if let Some(parent) = self.find_node_mut(parent_id) {
            if Self::add_child_to(parent, new_child) {
                self.rebuild_index();
                return true;
            }
        }
        false
    }

    /// Add a child to the root node.
    pub fn add_child_to_root(&mut self, new_child: LayoutNode) -> bool {
        if Self::add_child_to(&mut self.layout.root, new_child) {
            self.rebuild_index();
            return true;
        }
        false
    }

    /// Check if a node is a container that can accept children.
    pub fn is_container(&self, id: ComponentId) -> bool {
        if let Some(node) = self.find_node(id) {
            Self::node_is_container(node)
        } else {
            false
        }
    }

    /// Check if a node can accept children.
    fn node_is_container(node: &LayoutNode) -> bool {
        match &node.widget {
            crate::model::layout::WidgetType::Column { .. }
            | crate::model::layout::WidgetType::Row { .. }
            | crate::model::layout::WidgetType::Stack { .. } => true,
            crate::model::layout::WidgetType::Container { child, .. }
            | crate::model::layout::WidgetType::Scrollable { child, .. } => {
                // Single-child containers can only accept if empty
                child.is_none()
            }
            _ => false,
        }
    }

    /// Add a child to a specific node.
    fn add_child_to(node: &mut LayoutNode, new_child: LayoutNode) -> bool {
        match &mut node.widget {
            crate::model::layout::WidgetType::Column { children, .. }
            | crate::model::layout::WidgetType::Row { children, .. }
            | crate::model::layout::WidgetType::Stack { children, .. } => {
                children.push(new_child);
                true
            }
            crate::model::layout::WidgetType::Container { child, .. }
            | crate::model::layout::WidgetType::Scrollable { child, .. } => {
                if child.is_none() {
                    *child = Some(Box::new(new_child));
                    true
                } else {
                    false // Already has a child
                }
            }
            _ => false, // Not a container
        }
    }
}

/// Project templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Template {
    /// Empty layout with just a root Column.
    Blank,
    /// A form layout with text inputs and a submit button.
    Form,
    /// A dashboard layout with header and content panels.
    Dashboard,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::layout::{ButtonAttrs, ContainerAttrs, TextAttrs, WidgetType};
    use tempfile::tempdir;

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

    #[test]
    fn test_project_create_and_open() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        // Create a new project
        let created = Project::create(project_dir, None).unwrap();
        assert_eq!(created.layout.name, "Untitled");
        assert!(!created.dirty);

        // Verify files were created
        assert!(project_dir.join("iced_builder.toml").exists());
        assert!(project_dir.join("layout.ron").exists());

        // Re-open the project
        let opened = Project::open(project_dir).unwrap();
        assert_eq!(opened.layout.name, created.layout.name);
    }

    #[test]
    fn test_project_create_form_template() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        let project = Project::create(project_dir, Some(Template::Form)).unwrap();
        assert_eq!(project.layout.name, "Form");
        
        // Form template should have children (title, inputs, button)
        if let Some(children) = project.layout.root.children() {
            assert!(children.len() >= 3);
        } else {
            panic!("Form template root should have children");
        }
    }

    #[test]
    fn test_project_create_dashboard_template() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        let project = Project::create(project_dir, Some(Template::Dashboard)).unwrap();
        assert_eq!(project.layout.name, "Dashboard");
    }

    #[test]
    fn test_project_save() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        let mut project = Project::create(project_dir, None).unwrap();
        project.layout.name = "Test Layout".to_string();
        project.mark_dirty();
        assert!(project.dirty);

        project.save().unwrap();
        assert!(!project.dirty);

        // Re-open and verify
        let reopened = Project::open(project_dir).unwrap();
        assert_eq!(reopened.layout.name, "Test Layout");
    }

    #[test]
    fn test_project_export() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        // Create output directory
        std::fs::create_dir_all(project_dir.join("src/ui")).unwrap();

        let project = Project::create(project_dir, None).unwrap();
        let code = project.export().unwrap();

        assert!(code.contains("pub fn view"));
        assert!(code.contains("Element"));

        // Verify output file was created
        assert!(project_dir.join("src/ui/layout_generated.rs").exists());
    }

    #[test]
    fn test_project_find_node() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        let project = Project::create(project_dir, Some(Template::Form)).unwrap();
        
        // Should be able to find the root node
        let root_id = project.layout.root.id;
        let found = project.find_node(root_id);
        assert!(found.is_some());
    }

    #[test]
    fn test_project_open_missing_config() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        // Try to open a project without config
        let result = Project::open(project_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_project_remove_node() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        let mut project = Project::create(project_dir, Some(Template::Form)).unwrap();
        
        // Get the root and find a child to remove
        let root_id = project.layout.root.id;
        
        // Get a child node ID
        let child_id = project.layout.root.children()
            .expect("Root should be a container")
            .first()
            .expect("Should have at least one child")
            .id;
        
        // Cannot remove root
        assert!(!project.remove_node(root_id));
        
        // Can remove child
        assert!(project.remove_node(child_id));
        
        // Child should no longer be findable
        assert!(project.find_node(child_id).is_none());
    }

    #[test]
    fn test_project_remove_node_nested() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        let mut project = Project::create(project_dir, Some(Template::Dashboard)).unwrap();
        
        // Dashboard has nested structure, find a deeply nested node
        let children = project.layout.root.children().unwrap();
        if let Some(first_child) = children.first() {
            if let Some(nested_children) = first_child.children() {
                if let Some(nested_child) = nested_children.first() {
                    let nested_id = nested_child.id;
                    
                    // Remove the nested node
                    assert!(project.remove_node(nested_id));
                    
                    // Verify it's gone
                    assert!(project.find_node(nested_id).is_none());
                }
            }
        }
    }

    #[test]
    fn test_project_remove_nonexistent_node() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        let mut project = Project::create(project_dir, None).unwrap();
        
        // Try to remove a node that doesn't exist
        let fake_id = crate::model::layout::ComponentId::new();
        assert!(!project.remove_node(fake_id));
    }

    #[test]
    fn test_project_history_integration() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        let mut project = Project::create(project_dir, Some(Template::Form)).unwrap();
        
        // Initially no undo/redo available
        assert!(!project.history.can_undo());
        assert!(!project.history.can_redo());
        
        // Push a snapshot
        project.history.push(project.layout.clone());
        
        // Now undo should be available
        assert!(project.history.can_undo());
        assert!(!project.history.can_redo());
        
        // Get a child and modify
        let child_id = project.layout.root.children()
            .unwrap()
            .first()
            .unwrap()
            .id;
        
        // Remove the child
        project.remove_node(child_id);
        
        // Undo should restore the child
        let prev = project.history.undo(project.layout.clone()).unwrap();
        project.layout = prev;
        project.rebuild_index();
        
        // The child should be findable again
        assert!(project.find_node(child_id).is_some());
    }

    #[test]
    fn test_project_is_container() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        let mut project = Project::create(project_dir, Some(Template::Form)).unwrap();
        
        // Root should be a container (Column)
        let root_id = project.layout.root.id;
        assert!(project.is_container(root_id));
        
        // Add a non-container widget (Button) to root
        let button = LayoutNode::new(WidgetType::Button {
            label: "Test".to_string(),
            message_stub: "TestMsg".to_string(),
            attrs: ButtonAttrs::default(),
        });
        let button_id = button.id;
        assert!(project.add_child_to_root(button));
        
        // Button should not be a container
        assert!(!project.is_container(button_id));
    }

    #[test]
    fn test_project_add_child_to_root() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        let mut project = Project::create(project_dir, None).unwrap();
        
        let initial_count = project.layout.root.children().unwrap().len();
        
        // Add a text widget to root
        let text = LayoutNode::new(WidgetType::Text {
            content: "Hello".to_string(),
            attrs: TextAttrs::default(),
        });
        let text_id = text.id;
        assert!(project.add_child_to_root(text));
        
        // Verify it was added
        assert_eq!(
            project.layout.root.children().unwrap().len(),
            initial_count + 1
        );
        
        // Verify we can find it
        assert!(project.find_node(text_id).is_some());
    }

    #[test]
    fn test_project_add_child_to_node() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        let mut project = Project::create(project_dir, None).unwrap();
        
        // Add a Row container first
        let row = LayoutNode::new(WidgetType::Row {
            children: Vec::new(),
            attrs: ContainerAttrs::default(),
        });
        let row_id = row.id;
        assert!(project.add_child_to_root(row));
        
        // Now add a button to the Row
        let button = LayoutNode::new(WidgetType::Button {
            label: "Click".to_string(),
            message_stub: "Clicked".to_string(),
            attrs: ButtonAttrs::default(),
        });
        let button_id = button.id;
        assert!(project.add_child_to_node(row_id, button));
        
        // Verify button was added to the row
        let row_node = project.find_node(row_id).expect("Row should exist");
        let row_children = row_node.children().expect("Row should have children");
        assert!(row_children.iter().any(|c| c.id == button_id));
    }

    #[test]
    fn test_project_add_child_to_non_container() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        let mut project = Project::create(project_dir, None).unwrap();
        
        // Add a button first (non-container)
        let button = LayoutNode::new(WidgetType::Button {
            label: "Test".to_string(),
            message_stub: "TestMsg".to_string(),
            attrs: ButtonAttrs::default(),
        });
        let button_id = button.id;
        assert!(project.add_child_to_root(button));
        
        // Try to add a child to the button - should fail
        let text = LayoutNode::new(WidgetType::Text {
            content: "Hello".to_string(),
            attrs: TextAttrs::default(),
        });
        assert!(!project.add_child_to_node(button_id, text));
    }

    #[test]
    fn test_project_add_child_to_nonexistent_node() {
        let temp = tempdir().unwrap();
        let project_dir = temp.path();

        let mut project = Project::create(project_dir, None).unwrap();
        
        // Try to add to a non-existent node
        let fake_id = crate::model::layout::ComponentId::new();
        let text = LayoutNode::new(WidgetType::Text {
            content: "Hello".to_string(),
            attrs: TextAttrs::default(),
        });
        assert!(!project.add_child_to_node(fake_id, text));
    }
}
