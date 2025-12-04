//! Layout AST types representing the UI component tree.
//!
//! The Layout AST is an intermediate representation of the UI that can be:
//! - Edited visually in the builder
//! - Serialized to/from RON or JSON files
//! - Converted to Rust/Iced code

use crate::util::{is_rust_keyword, is_valid_rust_identifier};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

/// Unique identifier for a component in the layout tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(Uuid);

impl ComponentId {
    /// Create a new random component ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ComponentId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Length specification for width/height properties.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LengthSpec {
    /// Fill available space.
    Fill,
    /// Shrink to fit content.
    Shrink,
    /// Fill a portion of available space.
    FillPortion(u16),
    /// Fixed pixel size.
    Fixed(f32),
}

impl Default for LengthSpec {
    fn default() -> Self {
        Self::Shrink
    }
}

/// Alignment specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AlignmentSpec {
    #[default]
    Start,
    Center,
    End,
}

/// Padding specification (uniform or per-side).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct PaddingSpec {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl PaddingSpec {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    pub fn uniform(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

/// Common attributes for container widgets (Column, Row, Container, Scrollable).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerAttrs {
    pub padding: PaddingSpec,
    pub spacing: f32,
    pub align_x: AlignmentSpec,
    pub align_y: AlignmentSpec,
    pub width: LengthSpec,
    pub height: LengthSpec,
}

impl Default for ContainerAttrs {
    fn default() -> Self {
        Self {
            padding: PaddingSpec::ZERO,
            spacing: 0.0,
            align_x: AlignmentSpec::Start,
            align_y: AlignmentSpec::Start,
            width: LengthSpec::Shrink,
            height: LengthSpec::Shrink,
        }
    }
}

/// Attributes for Text widgets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextAttrs {
    pub font_size: f32,
    pub color: Option<[f32; 4]>, // RGBA, None means default
    pub horizontal_alignment: AlignmentSpec,
}

impl Default for TextAttrs {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            color: None,
            horizontal_alignment: AlignmentSpec::Start,
        }
    }
}

/// Attributes for Button widgets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ButtonAttrs {
    pub width: LengthSpec,
    pub height: LengthSpec,
}

/// Attributes for TextInput widgets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct InputAttrs {
    pub width: LengthSpec,
}

/// Attributes for Checkbox widgets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CheckboxAttrs {
    pub spacing: f32,
}

/// Attributes for Slider widgets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SliderAttrs {
    pub width: LengthSpec,
}

impl Default for SliderAttrs {
    fn default() -> Self {
        Self {
            width: LengthSpec::Fill,
        }
    }
}

/// Attributes for PickList widgets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PickListAttrs {
    pub width: LengthSpec,
    pub placeholder: String,
}

/// A node in the layout tree representing a widget or container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutNode {
    /// Unique identifier for this node.
    pub id: ComponentId,
    /// The widget type and its specific data.
    pub widget: WidgetType,
}

impl LayoutNode {
    /// Create a new layout node with a random ID.
    pub fn new(widget: WidgetType) -> Self {
        Self {
            id: ComponentId::new(),
            widget,
        }
    }

    /// Get children of this node (if it's a container).
    pub fn children(&self) -> Option<&Vec<LayoutNode>> {
        match &self.widget {
            WidgetType::Column { children, .. } => Some(children),
            WidgetType::Row { children, .. } => Some(children),
            _ => None,
        }
    }

    /// Get mutable children of this node (if it's a container).
    pub fn children_mut(&mut self) -> Option<&mut Vec<LayoutNode>> {
        match &mut self.widget {
            WidgetType::Column { children, .. } => Some(children),
            WidgetType::Row { children, .. } => Some(children),
            _ => None,
        }
    }
}

/// The type of widget and its associated data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WidgetType {
    /// A vertical container.
    Column {
        children: Vec<LayoutNode>,
        attrs: ContainerAttrs,
    },
    /// A horizontal container.
    Row {
        children: Vec<LayoutNode>,
        attrs: ContainerAttrs,
    },
    /// A single-child container for alignment/padding.
    Container {
        child: Option<Box<LayoutNode>>,
        attrs: ContainerAttrs,
    },
    /// A scrollable container.
    Scrollable {
        child: Option<Box<LayoutNode>>,
        attrs: ContainerAttrs,
    },
    /// A stack container for overlays.
    Stack {
        children: Vec<LayoutNode>,
        attrs: ContainerAttrs,
    },
    /// A text label.
    Text {
        content: String,
        attrs: TextAttrs,
    },
    /// A clickable button.
    Button {
        label: String,
        message_stub: String,
        attrs: ButtonAttrs,
    },
    /// A text input field.
    TextInput {
        placeholder: String,
        value_binding: String,
        message_stub: String,
        attrs: InputAttrs,
    },
    /// A checkbox.
    Checkbox {
        label: String,
        checked_binding: String,
        message_stub: String,
        attrs: CheckboxAttrs,
    },
    /// A slider.
    Slider {
        min: f32,
        max: f32,
        value_binding: String,
        message_stub: String,
        attrs: SliderAttrs,
    },
    /// A pick list (dropdown).
    PickList {
        options: Vec<String>,
        selected_binding: String,
        message_stub: String,
        attrs: PickListAttrs,
    },
    /// Empty space.
    Space {
        width: LengthSpec,
        height: LengthSpec,
    },
}

/// A complete layout document that can be saved/loaded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutDocument {
    /// Schema version for forward compatibility.
    pub version: u32,
    /// Human-readable name for this layout.
    pub name: String,
    /// The root node of the layout tree.
    pub root: LayoutNode,
}

impl Default for LayoutDocument {
    fn default() -> Self {
        Self {
            version: 1,
            name: String::from("Untitled"),
            root: LayoutNode::new(WidgetType::Column {
                children: Vec::new(),
                attrs: ContainerAttrs::default(),
            }),
        }
    }
}

// ============================================================================
// Validation
// ============================================================================

/// Severity level for validation issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// An error that must be fixed before code generation.
    Error,
    /// A warning that doesn't prevent code generation but may indicate a problem.
    Warning,
}

/// A validation error or warning found in the layout tree.
#[derive(Debug, Clone, Error)]
#[error("{severity:?} at {path}: {message}")]
pub struct ValidationError {
    /// The path to the node with the issue (e.g., "root.children[2].child").
    pub path: String,
    /// The severity of the issue.
    pub severity: ValidationSeverity,
    /// A human-readable description of the issue.
    pub message: String,
    /// The ComponentId of the node with the issue.
    pub node_id: ComponentId,
}

impl ValidationError {
    /// Create a new error.
    pub fn error(path: impl Into<String>, message: impl Into<String>, node_id: ComponentId) -> Self {
        Self {
            path: path.into(),
            severity: ValidationSeverity::Error,
            message: message.into(),
            node_id,
        }
    }

    /// Create a new warning.
    pub fn warning(path: impl Into<String>, message: impl Into<String>, node_id: ComponentId) -> Self {
        Self {
            path: path.into(),
            severity: ValidationSeverity::Warning,
            message: message.into(),
            node_id,
        }
    }
}

impl LayoutNode {
    /// Validate this node and its children.
    ///
    /// Returns a list of validation errors and warnings.
    pub fn validate(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        self.validate_recursive("root", &mut errors);
        errors
    }

    fn validate_recursive(&self, path: &str, errors: &mut Vec<ValidationError>) {
        // Check widget-specific constraints
        match &self.widget {
            // Multi-child containers
            WidgetType::Column { children, .. }
            | WidgetType::Row { children, .. }
            | WidgetType::Stack { children, .. } => {
                if children.is_empty() {
                    errors.push(ValidationError::warning(
                        path,
                        "Container has no children",
                        self.id,
                    ));
                }
                for (i, child) in children.iter().enumerate() {
                    let child_path = format!("{}.children[{}]", path, i);
                    child.validate_recursive(&child_path, errors);
                }
            }

            // Single-child containers
            WidgetType::Container { child, .. } | WidgetType::Scrollable { child, .. } => {
                if let Some(c) = child {
                    let child_path = format!("{}.child", path);
                    c.validate_recursive(&child_path, errors);
                } else {
                    errors.push(ValidationError::warning(
                        path,
                        "Container has no child",
                        self.id,
                    ));
                }
            }

            // Leaf widgets with bindings to validate
            WidgetType::Button { message_stub, .. } => {
                if !message_stub.is_empty() {
                    self.validate_identifier(path, "message_stub", message_stub, errors);
                }
            }
            WidgetType::TextInput { value_binding, message_stub, .. } => {
                if !value_binding.is_empty() {
                    self.validate_identifier(path, "value_binding", value_binding, errors);
                }
                if !message_stub.is_empty() {
                    self.validate_identifier(path, "message_stub", message_stub, errors);
                }
            }
            WidgetType::Checkbox { checked_binding, message_stub, .. } => {
                if !checked_binding.is_empty() {
                    self.validate_identifier(path, "checked_binding", checked_binding, errors);
                }
                if !message_stub.is_empty() {
                    self.validate_identifier(path, "message_stub", message_stub, errors);
                }
            }
            WidgetType::Slider { value_binding, message_stub, .. } => {
                if !value_binding.is_empty() {
                    self.validate_identifier(path, "value_binding", value_binding, errors);
                }
                if !message_stub.is_empty() {
                    self.validate_identifier(path, "message_stub", message_stub, errors);
                }
            }
            WidgetType::PickList { selected_binding, message_stub, .. } => {
                if !selected_binding.is_empty() {
                    self.validate_identifier(path, "selected_binding", selected_binding, errors);
                }
                if !message_stub.is_empty() {
                    self.validate_identifier(path, "message_stub", message_stub, errors);
                }
            }

            // Leaf widgets without special validation
            WidgetType::Text { .. } | WidgetType::Space { .. } => {}
        }
    }

    fn validate_identifier(&self, path: &str, field: &str, value: &str, errors: &mut Vec<ValidationError>) {
        if !is_valid_rust_identifier(value) {
            errors.push(ValidationError::error(
                path,
                format!("{} '{}' is not a valid Rust identifier", field, value),
                self.id,
            ));
        } else if is_rust_keyword(value) {
            errors.push(ValidationError::error(
                path,
                format!("{} '{}' is a Rust keyword and cannot be used as an identifier", field, value),
                self.id,
            ));
        }
    }
}

impl LayoutDocument {
    /// Validate the entire document.
    pub fn validate(&self) -> Vec<ValidationError> {
        self.root.validate()
    }

    /// Check if the document has any validation errors (not just warnings).
    pub fn has_errors(&self) -> bool {
        self.validate()
            .iter()
            .any(|e| e.severity == ValidationSeverity::Error)
    }
}

/// Index for O(1) node lookup by ComponentId.
pub type NodeIndex = HashMap<ComponentId, Vec<usize>>;

/// Build an index mapping ComponentIds to their paths in the tree.
pub fn build_node_index(root: &LayoutNode) -> NodeIndex {
    let mut index = HashMap::new();
    build_index_recursive(root, &mut Vec::new(), &mut index);
    index
}

fn build_index_recursive(node: &LayoutNode, path: &mut Vec<usize>, index: &mut NodeIndex) {
    index.insert(node.id, path.clone());

    if let Some(children) = node.children() {
        for (i, child) in children.iter().enumerate() {
            path.push(i);
            build_index_recursive(child, path, index);
            path.pop();
        }
    }

    // Handle single-child containers
    match &node.widget {
        WidgetType::Container { child: Some(c), .. }
        | WidgetType::Scrollable { child: Some(c), .. } => {
            path.push(0);
            build_index_recursive(c, path, index);
            path.pop();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_id_unique() {
        let id1 = ComponentId::new();
        let id2 = ComponentId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_layout_document_default() {
        let doc = LayoutDocument::default();
        assert_eq!(doc.version, 1);
        assert_eq!(doc.name, "Untitled");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let doc = LayoutDocument::default();
        let serialized = ron::to_string(&doc).unwrap();
        let deserialized: LayoutDocument = ron::from_str(&serialized).unwrap();
        assert_eq!(doc.name, deserialized.name);
    }

    #[test]
    fn test_serialization_json_roundtrip() {
        let doc = LayoutDocument::default();
        let serialized = serde_json::to_string(&doc).unwrap();
        let deserialized: LayoutDocument = serde_json::from_str(&serialized).unwrap();
        assert_eq!(doc.name, deserialized.name);
    }

    #[test]
    fn test_validate_empty_container_warning() {
        let doc = LayoutDocument::default();
        let errors = doc.validate();
        // Default document has an empty Column, should produce a warning
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].severity, ValidationSeverity::Warning);
        assert!(errors[0].message.contains("no children"));
    }

    #[test]
    fn test_validate_valid_identifier() {
        let mut doc = LayoutDocument::default();
        doc.root = LayoutNode::new(WidgetType::Button {
            label: "Click me".to_string(),
            message_stub: "handle_click".to_string(),
            attrs: ButtonAttrs::default(),
        });
        let errors = doc.validate();
        // Valid identifier, no errors
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_invalid_identifier() {
        let mut doc = LayoutDocument::default();
        doc.root = LayoutNode::new(WidgetType::Button {
            label: "Click me".to_string(),
            message_stub: "123-invalid".to_string(),
            attrs: ButtonAttrs::default(),
        });
        let errors = doc.validate();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].severity, ValidationSeverity::Error);
        assert!(errors[0].message.contains("not a valid Rust identifier"));
    }

    #[test]
    fn test_validate_rust_keyword() {
        let mut doc = LayoutDocument::default();
        doc.root = LayoutNode::new(WidgetType::Button {
            label: "Click me".to_string(),
            message_stub: "fn".to_string(),
            attrs: ButtonAttrs::default(),
        });
        let errors = doc.validate();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].severity, ValidationSeverity::Error);
        assert!(errors[0].message.contains("Rust keyword"));
    }

    #[test]
    fn test_validate_nested_containers() {
        let mut doc = LayoutDocument::default();
        doc.root = LayoutNode::new(WidgetType::Column {
            children: vec![
                LayoutNode::new(WidgetType::Row {
                    children: vec![],
                    attrs: ContainerAttrs::default(),
                }),
            ],
            attrs: ContainerAttrs::default(),
        });
        let errors = doc.validate();
        // The nested Row has no children, should produce a warning
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].severity, ValidationSeverity::Warning);
        assert!(errors[0].path.contains("children[0]"));
    }

    #[test]
    fn test_validate_text_input_bindings() {
        let mut doc = LayoutDocument::default();
        doc.root = LayoutNode::new(WidgetType::TextInput {
            placeholder: "Enter text".to_string(),
            value_binding: "user_input".to_string(),
            message_stub: "on-change".to_string(), // Invalid! Contains hyphen
            attrs: InputAttrs::default(),
        });
        let errors = doc.validate();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].severity, ValidationSeverity::Error);
        assert!(errors[0].message.contains("message_stub"));
    }

    #[test]
    fn test_has_errors() {
        let doc = LayoutDocument::default();
        // Empty container only produces warnings, not errors
        assert!(!doc.has_errors());

        let mut doc_with_error = LayoutDocument::default();
        doc_with_error.root = LayoutNode::new(WidgetType::Button {
            label: "Click".to_string(),
            message_stub: "123bad".to_string(),
            attrs: ButtonAttrs::default(),
        });
        assert!(doc_with_error.has_errors());
    }

    #[test]
    fn test_build_node_index() {
        let child1 = LayoutNode::new(WidgetType::Text {
            content: "Hello".to_string(),
            attrs: TextAttrs::default(),
        });
        let child1_id = child1.id;
        
        let child2 = LayoutNode::new(WidgetType::Button {
            label: "Click".to_string(),
            message_stub: String::new(),
            attrs: ButtonAttrs::default(),
        });
        let child2_id = child2.id;
        
        let root = LayoutNode::new(WidgetType::Column {
            children: vec![child1, child2],
            attrs: ContainerAttrs::default(),
        });
        let root_id = root.id;
        
        let index = build_node_index(&root);
        
        assert_eq!(index.get(&root_id), Some(&vec![]));
        assert_eq!(index.get(&child1_id), Some(&vec![0]));
        assert_eq!(index.get(&child2_id), Some(&vec![1]));
    }
}
