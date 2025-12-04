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

        let id_text = text(format!("ID: {}", node.id))
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
            WidgetType::Column { attrs, .. } | WidgetType::Row { attrs, .. } => {
                Self::render_container_props(attrs)
            }
            WidgetType::Container { attrs, .. } | WidgetType::Scrollable { attrs, .. } => {
                Self::render_container_props(attrs)
            }
            WidgetType::Text { content, attrs } => {
                Self::render_text_props(node.id, content, attrs)
            }
            WidgetType::Button { label, message_stub, .. } => {
                Self::render_button_props(node.id, label, message_stub)
            }
            WidgetType::TextInput { placeholder, value_binding, .. } => {
                Self::render_text_input_props(node.id, placeholder, value_binding)
            }
            _ => column![text("Properties not yet implemented").size(12)].into(),
        }
    }

    /// Render container properties (padding, spacing, alignment).
    fn render_container_props<'a>(
        attrs: &'a crate::model::layout::ContainerAttrs,
    ) -> Element<'a, Message> {
        let padding_str = format!("{}", attrs.padding.top);
        let spacing_str = format!("{}", attrs.spacing);
        
        column![
            Self::property_row_owned("Padding", padding_str),
            Self::property_row_owned("Spacing", spacing_str),
            Self::length_property("Width", attrs.width),
            Self::length_property("Height", attrs.height),
        ]
        .spacing(10)
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
            Self::labeled_input("Content", content, move |s| Message::UpdateTextContent(id, s)),
            Self::property_row_owned("Font Size", font_size_str),
        ]
        .spacing(10)
        .into()
    }

    /// Render button properties.
    fn render_button_props<'a>(
        id: ComponentId,
        label: &'a str,
        message_stub: &'a str,
    ) -> Element<'a, Message> {
        column![
            Self::labeled_input("Label", label, move |s| Message::UpdateButtonLabel(id, s)),
            Self::labeled_input("Message", message_stub, move |s| Message::UpdateMessageStub(id, s)),
        ]
        .spacing(10)
        .into()
    }

    /// Render text input properties.
    fn render_text_input_props<'a>(
        id: ComponentId,
        placeholder: &'a str,
        value_binding: &'a str,
    ) -> Element<'a, Message> {
        column![
            Self::labeled_input("Placeholder", placeholder, move |s| Message::UpdatePlaceholder(id, s)),
            Self::labeled_input("Binding", value_binding, move |s| Message::UpdateBinding(id, s)),
        ]
        .spacing(10)
        .into()
    }

    /// Render a simple property row with owned value.
    fn property_row_owned<'a>(label: &'static str, value: String) -> Column<'a, Message> {
        column![
            text(label).size(11).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            text(value).size(13),
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

    /// Render a length property with picker.
    fn length_property<'a>(label: &'static str, value: LengthSpec) -> Column<'a, Message> {
        let display = match value {
            LengthSpec::Fill => "Fill",
            LengthSpec::Shrink => "Shrink",
            LengthSpec::FillPortion(_) => "FillPortion",
            LengthSpec::Fixed(_) => "Fixed",
        };

        column![
            text(label).size(11).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            text(display).size(13),
        ]
        .spacing(2)
    }
}
