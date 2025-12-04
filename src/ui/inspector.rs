//! Property inspector sidebar.
//!
//! Displays and allows editing of properties for the selected component.

use iced::widget::{column, container, scrollable, text, text_input, Column};
use iced::{Element, Length};

use crate::app::Message;
use crate::model::{
    layout::{LengthSpec, WidgetType},
    ComponentId, LayoutNode,
};

/// The property inspector component.
pub struct Inspector;

impl Inspector {
    /// Render the inspector with properties for the selected node.
    pub fn view<'a>(
        selected_node: Option<&'a LayoutNode>,
        _selected_id: Option<ComponentId>,
    ) -> Element<'a, Message> {
        let content: Element<'a, Message> = match selected_node {
            Some(node) => Self::render_properties(node),
            None => Self::render_empty(),
        };

        container(scrollable(content).height(Length::Fill))
            .width(Length::Fixed(250.0))
            .height(Length::Fill)
            .padding(10)
            .into()
    }

    /// Render the empty state when nothing is selected.
    fn render_empty<'a>() -> Element<'a, Message> {
        text("Select a component to edit its properties.")
            .size(13)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5))
            .into()
    }

    /// Render properties for the selected node.
    fn render_properties<'a>(node: &'a LayoutNode) -> Element<'a, Message> {
        let header = text(Self::widget_type_name(&node.widget))
            .size(16);

        let id_text = text(format!("ID: {}...", &node.id.to_string()[..8]))
            .size(11)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5));

        let properties = Self::render_widget_properties(node);

        column![header, id_text, properties]
            .spacing(15)
            .into()
    }

    /// Get the display name for a widget type.
    fn widget_type_name(widget: &WidgetType) -> &'static str {
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

    /// Render properties specific to the widget type.
    fn render_widget_properties<'a>(node: &'a LayoutNode) -> Element<'a, Message> {
        match &node.widget {
            WidgetType::Column { attrs, children } | WidgetType::Row { attrs, children } => {
                Self::render_container_props(node.id, attrs, Some(children.len()))
            }
            WidgetType::Container { attrs, child } => {
                Self::render_container_props(node.id, attrs, child.as_ref().map(|_| 1))
            }
            WidgetType::Scrollable { attrs, child } => {
                Self::render_container_props(node.id, attrs, child.as_ref().map(|_| 1))
            }
            WidgetType::Stack { attrs, children } => {
                Self::render_container_props(node.id, attrs, Some(children.len()))
            }
            WidgetType::Text { content, attrs } => {
                Self::render_text_props(node.id, content, attrs)
            }
            WidgetType::Button { label, message_stub, .. } => {
                Self::render_button_props(node.id, label, message_stub)
            }
            WidgetType::TextInput { placeholder, value_binding, message_stub, .. } => {
                Self::render_text_input_props(node.id, placeholder, value_binding, message_stub)
            }
            WidgetType::Checkbox { label, checked_binding, message_stub, .. } => {
                Self::render_checkbox_props(node.id, label, checked_binding, message_stub)
            }
            WidgetType::Slider { min, max, value_binding, message_stub, .. } => {
                Self::render_slider_props(node.id, *min, *max, value_binding, message_stub)
            }
            WidgetType::PickList { options, selected_binding, message_stub, .. } => {
                Self::render_picklist_props(node.id, options, selected_binding, message_stub)
            }
            WidgetType::Space { width, height } => {
                Self::render_space_props(*width, *height)
            }
        }
    }

    /// Render container properties (padding, spacing, alignment).
    fn render_container_props(
        id: ComponentId,
        attrs: &crate::model::layout::ContainerAttrs,
        child_count: Option<usize>,
    ) -> Element<'static, Message> {
        let padding_str = format!("{}", attrs.padding.top);
        let spacing_str = format!("{}", attrs.spacing);
        let children_text = match child_count {
            Some(n) => format!("{} children", n),
            None => "No child".to_string(),
        };
        let width_display = Self::length_display(attrs.width).to_string();
        let height_display = Self::length_display(attrs.height).to_string();
        
        column![
            Self::section_header("Layout"),
            Self::numeric_input_owned("Padding", padding_str, move |s| {
                s.parse::<f32>().ok().map(|v| Message::UpdatePadding(id, v)).unwrap_or(Message::Noop)
            }),
            Self::numeric_input_owned("Spacing", spacing_str, move |s| {
                s.parse::<f32>().ok().map(|v| Message::UpdateSpacing(id, v)).unwrap_or(Message::Noop)
            }),
            Self::property_row_owned("Width", width_display),
            Self::property_row_owned("Height", height_display),
            Self::section_header("Content"),
            Self::property_row_owned("Children", children_text),
        ]
        .spacing(8)
        .into()
    }

    /// Render text properties.
    fn render_text_props<'a>(
        id: ComponentId,
        content: &'a str,
        attrs: &'a crate::model::layout::TextAttrs,
    ) -> Element<'a, Message> {
        let font_size_str = format!("{}", attrs.font_size);
        
        column![
            Self::section_header("Content"),
            Self::labeled_input("Text", content, move |s| Message::UpdateTextContent(id, s)),
            Self::section_header("Style"),
            Self::property_row_static("Font Size", &font_size_str),
            Self::property_row_static("Alignment", Self::alignment_display(attrs.horizontal_alignment)),
        ]
        .spacing(8)
        .into()
    }

    /// Render button properties.
    fn render_button_props<'a>(
        id: ComponentId,
        label: &'a str,
        message_stub: &'a str,
    ) -> Element<'a, Message> {
        column![
            Self::section_header("Content"),
            Self::labeled_input("Label", label, move |s| Message::UpdateButtonLabel(id, s)),
            Self::section_header("Interaction"),
            Self::labeled_input("Message", message_stub, move |s| Message::UpdateMessageStub(id, s)),
        ]
        .spacing(8)
        .into()
    }

    /// Render text input properties.
    fn render_text_input_props<'a>(
        id: ComponentId,
        placeholder: &'a str,
        value_binding: &'a str,
        message_stub: &'a str,
    ) -> Element<'a, Message> {
        column![
            Self::section_header("Content"),
            Self::labeled_input("Placeholder", placeholder, move |s| Message::UpdatePlaceholder(id, s)),
            Self::section_header("Bindings"),
            Self::labeled_input("Value Binding", value_binding, move |s| Message::UpdateBinding(id, s.clone())),
            Self::labeled_input("Message", message_stub, move |s| Message::UpdateMessageStub(id, s)),
        ]
        .spacing(8)
        .into()
    }

    /// Render checkbox properties.
    fn render_checkbox_props<'a>(
        id: ComponentId,
        label: &'a str,
        checked_binding: &'a str,
        message_stub: &'a str,
    ) -> Element<'a, Message> {
        column![
            Self::section_header("Content"),
            Self::labeled_input("Label", label, move |s| Message::UpdateCheckboxLabel(id, s)),
            Self::section_header("Bindings"),
            Self::labeled_input("Checked Binding", checked_binding, move |s| Message::UpdateBinding(id, s.clone())),
            Self::labeled_input("Message", message_stub, move |s| Message::UpdateMessageStub(id, s)),
        ]
        .spacing(8)
        .into()
    }

    /// Render slider properties.
    fn render_slider_props<'a>(
        id: ComponentId,
        min: f32,
        max: f32,
        value_binding: &'a str,
        message_stub: &'a str,
    ) -> Element<'a, Message> {
        let min_str = format!("{}", min);
        let max_str = format!("{}", max);
        
        column![
            Self::section_header("Range"),
            Self::property_row_static("Min", &min_str),
            Self::property_row_static("Max", &max_str),
            Self::section_header("Bindings"),
            Self::labeled_input("Value Binding", value_binding, move |s| Message::UpdateBinding(id, s.clone())),
            Self::labeled_input("Message", message_stub, move |s| Message::UpdateMessageStub(id, s)),
        ]
        .spacing(8)
        .into()
    }

    /// Render picklist properties.
    fn render_picklist_props<'a>(
        id: ComponentId,
        options: &'a [String],
        selected_binding: &'a str,
        message_stub: &'a str,
    ) -> Element<'a, Message> {
        let options_str = format!("{} options", options.len());
        
        column![
            Self::section_header("Options"),
            Self::property_row_static("Count", &options_str),
            Self::section_header("Bindings"),
            Self::labeled_input("Selected Binding", selected_binding, move |s| Message::UpdateBinding(id, s.clone())),
            Self::labeled_input("Message", message_stub, move |s| Message::UpdateMessageStub(id, s)),
        ]
        .spacing(8)
        .into()
    }

    /// Render space properties.
    fn render_space_props<'a>(width: LengthSpec, height: LengthSpec) -> Element<'a, Message> {
        column![
            Self::section_header("Dimensions"),
            Self::property_row_static("Width", Self::length_display(width)),
            Self::property_row_static("Height", Self::length_display(height)),
        ]
        .spacing(8)
        .into()
    }

    /// Render a section header.
    fn section_header<'a>(title: &'static str) -> Column<'a, Message> {
        column![
            text(title)
                .size(12)
                .color(iced::Color::from_rgb(0.4, 0.6, 0.9)),
        ]
    }

    /// Render a property row with owned value.
    fn property_row_owned(label: &'static str, value: String) -> Column<'static, Message> {
        column![
            text(label).size(11).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            text(value).size(13),
        ]
        .spacing(2)
    }

    /// Render a property row with static value.
    fn property_row_static<'a>(label: &'static str, value: &str) -> Column<'a, Message> {
        column![
            text(label).size(11).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            text(value.to_string()).size(13),
        ]
        .spacing(2)
    }

    /// Render a numeric input with owned value.
    fn numeric_input_owned<F>(label: &'static str, value: String, on_change: F) -> Column<'static, Message>
    where
        F: Fn(String) -> Message + 'static,
    {
        column![
            text(label).size(11).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            text_input("", &value)
                .on_input(on_change)
                .size(13),
        ]
        .spacing(2)
    }

    /// Render a numeric input that parses to f32.
    #[allow(dead_code)]
    fn numeric_input<'a, F>(label: &'static str, value: &'a str, on_change: F) -> Column<'a, Message>
    where
        F: Fn(String) -> Message + 'a,
    {
        column![
            text(label).size(11).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            text_input("", value)
                .on_input(on_change)
                .size(13),
        ]
        .spacing(2)
    }

    /// Render a labeled text input.
    fn labeled_input<'a, F>(label: &'static str, value: &'a str, on_change: F) -> Column<'a, Message>
    where
        F: Fn(String) -> Message + 'a,
    {
        column![
            text(label).size(11).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            text_input("", value)
                .on_input(on_change)
                .size(13),
        ]
        .spacing(2)
    }

    /// Get display string for a LengthSpec.
    fn length_display(value: LengthSpec) -> &'static str {
        match value {
            LengthSpec::Fill => "Fill",
            LengthSpec::Shrink => "Shrink",
            LengthSpec::FillPortion(_) => "FillPortion",
            LengthSpec::Fixed(_) => "Fixed",
        }
    }

    /// Get display string for an AlignmentSpec.
    fn alignment_display(value: crate::model::layout::AlignmentSpec) -> &'static str {
        match value {
            crate::model::layout::AlignmentSpec::Start => "Start",
            crate::model::layout::AlignmentSpec::Center => "Center",
            crate::model::layout::AlignmentSpec::End => "End",
        }
    }
}
