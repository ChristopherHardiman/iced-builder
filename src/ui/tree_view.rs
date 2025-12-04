//! Tree view for hierarchical layout navigation.
//!
//! Displays the component tree in a collapsible, hierarchical format
//! similar to a DOM inspector.

use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Color, Element, Length};

use crate::app::Message;
use crate::model::{layout::WidgetType, ComponentId, LayoutNode};

/// The tree view component.
pub struct TreeView;

impl TreeView {
    /// Render the tree view.
    pub fn view<'a>(
        root: &'a LayoutNode,
        selected_id: Option<ComponentId>,
    ) -> Element<'a, Message> {
        let content = Self::render_node(root, selected_id, 0);

        container(scrollable(
            container(content).padding(10).width(Length::Fill),
        ).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fixed(200.0))
        .into()
    }

    /// Recursively render a node and its children.
    fn render_node<'a>(
        node: &'a LayoutNode,
        selected_id: Option<ComponentId>,
        depth: usize,
    ) -> Element<'a, Message> {
        let is_selected = selected_id == Some(node.id);
        let indent = Space::new(Length::Fixed((depth * 16) as f32), Length::Shrink);

        let icon = Self::get_icon(&node.widget);
        let name = Self::get_name(&node.widget);

        let label_color = if is_selected {
            Color::from_rgb(0.3, 0.7, 1.0)
        } else {
            Color::from_rgb(0.8, 0.8, 0.8)
        };

        let node_row = row![
            indent,
            text(icon).size(12),
            Space::new(Length::Fixed(4.0), Length::Shrink),
            button(text(name).size(12).color(label_color))
                .on_press(Message::SelectComponent(node.id))
                .padding(2)
                .style(|_theme, _status| button::Style {
                    background: None,
                    ..Default::default()
                }),
        ]
        .align_y(iced::Alignment::Center);

        // Render children
        let children = Self::get_children(node);
        if children.is_empty() {
            node_row.into()
        } else {
            let mut col = column![node_row].spacing(2);
            for child in children {
                col = col.push(Self::render_node(child, selected_id, depth + 1));
            }
            col.into()
        }
    }

    /// Get an icon character for the widget type.
    fn get_icon(widget: &WidgetType) -> &'static str {
        match widget {
            WidgetType::Column { .. } => "┃",
            WidgetType::Row { .. } => "━",
            WidgetType::Container { .. } => "□",
            WidgetType::Scrollable { .. } => "⬍",
            WidgetType::Stack { .. } => "▤",
            WidgetType::Text { .. } => "T",
            WidgetType::Button { .. } => "◉",
            WidgetType::TextInput { .. } => "▭",
            WidgetType::Checkbox { .. } => "☑",
            WidgetType::Slider { .. } => "─●",
            WidgetType::PickList { .. } => "▼",
            WidgetType::Space { .. } => "·",
        }
    }

    /// Get a display name for the widget.
    fn get_name(widget: &WidgetType) -> &'static str {
        match widget {
            WidgetType::Column { .. } => "Column",
            WidgetType::Row { .. } => "Row",
            WidgetType::Container { .. } => "Container",
            WidgetType::Scrollable { .. } => "Scrollable",
            WidgetType::Stack { .. } => "Stack",
            WidgetType::Text { .. } => "Text",
            WidgetType::Button { .. } => "Button",
            WidgetType::TextInput { .. } => "TextInput",
            WidgetType::Checkbox { .. } => "Checkbox",
            WidgetType::Slider { .. } => "Slider",
            WidgetType::PickList { .. } => "PickList",
            WidgetType::Space { .. } => "Space",
        }
    }

    /// Get children of a node.
    fn get_children(node: &LayoutNode) -> Vec<&LayoutNode> {
        match &node.widget {
            WidgetType::Column { children, .. }
            | WidgetType::Row { children, .. }
            | WidgetType::Stack { children, .. } => children.iter().collect(),
            WidgetType::Container { child, .. } | WidgetType::Scrollable { child, .. } => {
                child.as_ref().map(|c| vec![c.as_ref()]).unwrap_or_default()
            }
            _ => Vec::new(),
        }
    }
}
