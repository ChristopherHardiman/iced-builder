//! Layout AST types representing the UI component tree.
//!
//! The Layout AST is an intermediate representation of the UI that can be:
//! - Edited visually in the builder
//! - Serialized to/from RON or JSON files
//! - Converted to Rust/Iced code

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
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
}
