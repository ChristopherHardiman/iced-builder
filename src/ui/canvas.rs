//! Canvas/viewport for rendering and interacting with the layout.
//!
//! Renders the layout tree using actual Iced widgets wrapped in MouseArea
//! for click interception and selection.

use iced::widget::{
    button, center, checkbox, column, container, mouse_area, row, scrollable, slider, text,
    text_input, Space,
};
use iced::{Border, Color, Element, Length};

use crate::app::Message;
use crate::model::{
    layout::{AlignmentSpec, LengthSpec, WidgetType},
    ComponentId, LayoutNode,
};

/// The canvas component for rendering and editing the layout.
pub struct Canvas;

impl Canvas {
    /// Render the canvas with the given layout.
    pub fn view<'a>(
        root: &'a LayoutNode,
        selected_id: Option<ComponentId>,
    ) -> Element<'a, Message> {
        let content = Self::render_node(root, selected_id);

        container(scrollable(container(content).padding(20).width(Length::Fill)))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
                ..Default::default()
            })
            .into()
    }

    /// Render an empty canvas placeholder.
    pub fn view_empty<'a>() -> Element<'a, Message> {
        container(center(
            text("No project open.\nUse File → New or File → Open to get started.")
                .size(16)
                .color(Color::from_rgb(0.5, 0.5, 0.5)),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
            ..Default::default()
        })
        .into()
    }

    /// Recursively render a layout node.
    fn render_node<'a>(node: &'a LayoutNode, selected_id: Option<ComponentId>) -> Element<'a, Message> {
        let is_selected = selected_id == Some(node.id);
        let widget = Self::render_widget(node, selected_id);

        // Wrap in mouse_area for selection
        let selectable = mouse_area(widget).on_press(Message::SelectComponent(node.id));

        // Apply selection styling if selected
        if is_selected {
            container(selectable)
                .style(|_theme| container::Style {
                    border: Border {
                        color: Color::from_rgb(0.2, 0.6, 1.0),
                        width: 2.0,
                        radius: 4.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        } else {
            selectable.into()
        }
    }

    /// Render the actual widget based on its type.
    fn render_widget<'a>(node: &'a LayoutNode, selected_id: Option<ComponentId>) -> Element<'a, Message> {
        match &node.widget {
            WidgetType::Column { children, attrs } => {
                let mut col = column![];
                for child in children {
                    col = col.push(Self::render_node(child, selected_id));
                }
                col.spacing(attrs.spacing)
                    .padding(iced::Padding::new(attrs.padding.top)
                        .right(attrs.padding.right)
                        .bottom(attrs.padding.bottom)
                        .left(attrs.padding.left))
                    .width(Self::convert_length(attrs.width))
                    .height(Self::convert_length(attrs.height))
                    .into()
            }

            WidgetType::Row { children, attrs } => {
                let mut r = row![];
                for child in children {
                    r = r.push(Self::render_node(child, selected_id));
                }
                r.spacing(attrs.spacing)
                    .padding(iced::Padding::new(attrs.padding.top)
                        .right(attrs.padding.right)
                        .bottom(attrs.padding.bottom)
                        .left(attrs.padding.left))
                    .width(Self::convert_length(attrs.width))
                    .height(Self::convert_length(attrs.height))
                    .into()
            }

            WidgetType::Container { child, attrs } => {
                let content: Element<'a, Message> = match child {
                    Some(c) => Self::render_node(c, selected_id),
                    None => text("(empty)").color(Color::from_rgb(0.5, 0.5, 0.5)).into(),
                };
                container(content)
                    .padding(iced::Padding::new(attrs.padding.top)
                        .right(attrs.padding.right)
                        .bottom(attrs.padding.bottom)
                        .left(attrs.padding.left))
                    .width(Self::convert_length(attrs.width))
                    .height(Self::convert_length(attrs.height))
                    .into()
            }

            WidgetType::Scrollable { child, attrs } => {
                let content: Element<'a, Message> = match child {
                    Some(c) => Self::render_node(c, selected_id),
                    None => text("(empty)").color(Color::from_rgb(0.5, 0.5, 0.5)).into(),
                };
                scrollable(content)
                    .width(Self::convert_length(attrs.width))
                    .height(Self::convert_length(attrs.height))
                    .into()
            }

            WidgetType::Stack { children, attrs } => {
                // For now, just render children in a column (Stack requires more complex handling)
                let mut col = column![];
                for child in children {
                    col = col.push(Self::render_node(child, selected_id));
                }
                col.width(Self::convert_length(attrs.width))
                    .height(Self::convert_length(attrs.height))
                    .into()
            }

            WidgetType::Text { content, attrs } => {
                let mut t = text(content.as_str()).size(attrs.font_size);
                if let Some(color) = attrs.color {
                    t = t.color(Color::from_rgba(color[0], color[1], color[2], color[3]));
                }
                t.into()
            }

            WidgetType::Button { label, .. } => {
                // In design mode, buttons always select instead of firing their action
                button(text(label.as_str()))
                    .on_press(Message::SelectComponent(node.id))
                    .into()
            }

            WidgetType::TextInput { placeholder, .. } => {
                // In design mode, text inputs are read-only
                text_input(placeholder.as_str(), "")
                    .into()
            }

            WidgetType::Checkbox { label, .. } => {
                // In design mode, checkboxes don't toggle
                checkbox(label.as_str(), false).into()
            }

            WidgetType::Slider { min, max, .. } => {
                // In design mode, sliders don't change
                let mid = (min + max) / 2.0;
                slider(*min..=*max, mid, |_| Message::Noop).into()
            }

            WidgetType::PickList { options, attrs, .. } => {
                // Show as a disabled-looking text for now
                let display = if options.is_empty() {
                    attrs.placeholder.as_str()
                } else {
                    &options[0]
                };
                container(text(display).size(14))
                    .padding(5)
                    .style(|_theme| container::Style {
                        border: Border {
                            color: Color::from_rgb(0.4, 0.4, 0.4),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    })
                    .into()
            }

            WidgetType::Space { width, height } => {
                Space::new(Self::convert_length(*width), Self::convert_length(*height)).into()
            }
        }
    }

    /// Convert LengthSpec to Iced Length.
    fn convert_length(spec: LengthSpec) -> Length {
        match spec {
            LengthSpec::Fill => Length::Fill,
            LengthSpec::Shrink => Length::Shrink,
            LengthSpec::FillPortion(p) => Length::FillPortion(p),
            LengthSpec::Fixed(f) => Length::Fixed(f),
        }
    }

    /// Convert AlignmentSpec to Iced Alignment.
    #[allow(dead_code)]
    fn convert_alignment(spec: AlignmentSpec) -> iced::Alignment {
        match spec {
            AlignmentSpec::Start => iced::Alignment::Start,
            AlignmentSpec::Center => iced::Alignment::Center,
            AlignmentSpec::End => iced::Alignment::End,
        }
    }
}
