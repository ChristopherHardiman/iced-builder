//! Canvas/viewport for rendering and interacting with the layout.
//!
//! Renders the layout tree using actual Iced widgets wrapped in MouseArea
//! for click interception and selection.

use iced::widget::{
    button, center, checkbox, column, container, mouse_area, row, scrollable, slider, stack, text,
    text_input, Space,
};
use iced::{Border, Color, Element, Length};

use crate::app::{EditorMode, Message};
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
        mode: EditorMode,
    ) -> Element<'a, Message> {
        // Render the root node, but override height to Shrink for scrollable compatibility
        let content = Self::render_node_for_canvas(root, selected_id, true, mode);

        let background_color = match mode {
            EditorMode::Design => Color::from_rgb(0.15, 0.15, 0.15),
            EditorMode::Preview => Color::from_rgb(0.1, 0.1, 0.12), // Slightly different for preview
        };

        container(scrollable(container(content).padding(20).width(Length::Fill)))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(background_color)),
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

    /// Render a node for the canvas, with special handling for the root node.
    /// The root node's height is forced to Shrink to work inside a scrollable.
    fn render_node_for_canvas<'a>(
        node: &'a LayoutNode,
        selected_id: Option<ComponentId>,
        is_root: bool,
        mode: EditorMode,
    ) -> Element<'a, Message> {
        let is_selected = selected_id == Some(node.id);
        let widget = Self::render_widget_for_canvas(node, selected_id, is_root, mode);

        // In design mode, wrap in mouse_area for selection
        // In preview mode, don't wrap (let widgets behave normally)
        let wrapped: Element<'a, Message> = match mode {
            EditorMode::Design => {
                mouse_area(widget).on_press(Message::SelectComponent(node.id)).into()
            }
            EditorMode::Preview => widget,
        };

        // Apply selection styling if selected (only in design mode)
        if is_selected && mode == EditorMode::Design {
            container(wrapped)
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
            wrapped
        }
    }

    /// Recursively render a layout node.
    fn render_node<'a>(node: &'a LayoutNode, selected_id: Option<ComponentId>, mode: EditorMode) -> Element<'a, Message> {
        let is_selected = selected_id == Some(node.id);
        let widget = Self::render_widget(node, selected_id, mode);

        // In design mode, wrap in mouse_area for selection
        let wrapped: Element<'a, Message> = match mode {
            EditorMode::Design => {
                mouse_area(widget).on_press(Message::SelectComponent(node.id)).into()
            }
            EditorMode::Preview => widget,
        };

        // Apply selection styling if selected (only in design mode)
        if is_selected && mode == EditorMode::Design {
            container(wrapped)
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
            wrapped
        }
    }

    /// Render widget for canvas root - forces height to Shrink for scrollable compatibility.
    fn render_widget_for_canvas<'a>(
        node: &'a LayoutNode,
        selected_id: Option<ComponentId>,
        is_root: bool,
        mode: EditorMode,
    ) -> Element<'a, Message> {
        match &node.widget {
            WidgetType::Column { children, attrs } => {
                let mut col = column![];
                for child in children {
                    col = col.push(Self::render_node(child, selected_id, mode));
                }
                // For root node, use Shrink height to work inside scrollable
                let height = if is_root {
                    Length::Shrink
                } else {
                    Self::convert_length(attrs.height)
                };
                col.spacing(attrs.spacing)
                    .padding(iced::Padding::new(attrs.padding.top)
                        .right(attrs.padding.right)
                        .bottom(attrs.padding.bottom)
                        .left(attrs.padding.left))
                    .width(Self::convert_length(attrs.width))
                    .height(height)
                    .align_x(Self::convert_horizontal_alignment(attrs.align_x))
                    .into()
            }

            WidgetType::Row { children, attrs } => {
                let mut r = row![];
                for child in children {
                    r = r.push(Self::render_node(child, selected_id, mode));
                }
                let height = if is_root {
                    Length::Shrink
                } else {
                    Self::convert_length(attrs.height)
                };
                r.spacing(attrs.spacing)
                    .padding(iced::Padding::new(attrs.padding.top)
                        .right(attrs.padding.right)
                        .bottom(attrs.padding.bottom)
                        .left(attrs.padding.left))
                    .width(Self::convert_length(attrs.width))
                    .height(height)
                    .align_y(Self::convert_vertical_alignment(attrs.align_y))
                    .into()
            }

            // For other widget types, delegate to render_widget
            _ => Self::render_widget(node, selected_id, mode),
        }
    }

    /// Render the actual widget based on its type.
    fn render_widget<'a>(node: &'a LayoutNode, selected_id: Option<ComponentId>, mode: EditorMode) -> Element<'a, Message> {
        match &node.widget {
            WidgetType::Column { children, attrs } => {
                let mut col = column![];
                for child in children {
                    col = col.push(Self::render_node(child, selected_id, mode));
                }
                col.spacing(attrs.spacing)
                    .padding(iced::Padding::new(attrs.padding.top)
                        .right(attrs.padding.right)
                        .bottom(attrs.padding.bottom)
                        .left(attrs.padding.left))
                    .width(Self::convert_length(attrs.width))
                    .height(Self::convert_length(attrs.height))
                    .align_x(Self::convert_horizontal_alignment(attrs.align_x))
                    .into()
            }

            WidgetType::Row { children, attrs } => {
                let mut r = row![];
                for child in children {
                    r = r.push(Self::render_node(child, selected_id, mode));
                }
                r.spacing(attrs.spacing)
                    .padding(iced::Padding::new(attrs.padding.top)
                        .right(attrs.padding.right)
                        .bottom(attrs.padding.bottom)
                        .left(attrs.padding.left))
                    .width(Self::convert_length(attrs.width))
                    .height(Self::convert_length(attrs.height))
                    .align_y(Self::convert_vertical_alignment(attrs.align_y))
                    .into()
            }

            WidgetType::Container { child, attrs } => {
                let content: Element<'a, Message> = match child {
                    Some(c) => Self::render_node(c, selected_id, mode),
                    None => text("(empty)").color(Color::from_rgb(0.5, 0.5, 0.5)).into(),
                };
                container(content)
                    .padding(iced::Padding::new(attrs.padding.top)
                        .right(attrs.padding.right)
                        .bottom(attrs.padding.bottom)
                        .left(attrs.padding.left))
                    .width(Self::convert_length(attrs.width))
                    .height(Self::convert_length(attrs.height))
                    .align_x(Self::convert_horizontal_alignment(attrs.align_x))
                    .align_y(Self::convert_vertical_alignment(attrs.align_y))
                    .into()
            }

            WidgetType::Scrollable { child, attrs } => {
                let content: Element<'a, Message> = match child {
                    Some(c) => Self::render_node(c, selected_id, mode),
                    None => text("(empty)").color(Color::from_rgb(0.5, 0.5, 0.5)).into(),
                };
                scrollable(content)
                    .width(Self::convert_length(attrs.width))
                    .height(Self::convert_length(attrs.height))
                    .into()
            }

            WidgetType::Stack { children, attrs } => {
                // Use Iced's stack widget for overlays
                let layers: Vec<Element<'a, Message>> = children
                    .iter()
                    .map(|child| Self::render_node(child, selected_id, mode))
                    .collect();
                
                stack(layers)
                    .width(Self::convert_length(attrs.width))
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
                match mode {
                    EditorMode::Design => {
                        // In design mode, buttons select instead of firing their action
                        button(text(label.as_str()))
                            .on_press(Message::SelectComponent(node.id))
                            .into()
                    }
                    EditorMode::Preview => {
                        // In preview mode, buttons show as clickable but don't do anything
                        button(text(label.as_str()))
                            .on_press(Message::Noop)
                            .into()
                    }
                }
            }

            WidgetType::TextInput { placeholder, .. } => {
                match mode {
                    EditorMode::Design => {
                        // In design mode, text inputs are read-only
                        text_input(placeholder.as_str(), "")
                            .into()
                    }
                    EditorMode::Preview => {
                        // In preview mode, text inputs can be typed into (but changes aren't saved)
                        text_input(placeholder.as_str(), "")
                            .on_input(|_| Message::Noop)
                            .into()
                    }
                }
            }

            WidgetType::Checkbox { label, .. } => {
                match mode {
                    EditorMode::Design => {
                        // In design mode, checkboxes don't toggle
                        checkbox(label.as_str(), false).into()
                    }
                    EditorMode::Preview => {
                        // In preview mode, checkboxes can be toggled (but state isn't saved)
                        checkbox(label.as_str(), false)
                            .on_toggle(|_| Message::Noop)
                            .into()
                    }
                }
            }

            WidgetType::Slider { min, max, .. } => {
                // In both modes, sliders show at midpoint
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

    /// Convert AlignmentSpec to Iced Horizontal alignment.
    fn convert_horizontal_alignment(spec: AlignmentSpec) -> iced::alignment::Horizontal {
        match spec {
            AlignmentSpec::Start => iced::alignment::Horizontal::Left,
            AlignmentSpec::Center => iced::alignment::Horizontal::Center,
            AlignmentSpec::End => iced::alignment::Horizontal::Right,
        }
    }

    /// Convert AlignmentSpec to Iced Vertical alignment.
    fn convert_vertical_alignment(spec: AlignmentSpec) -> iced::alignment::Vertical {
        match spec {
            AlignmentSpec::Start => iced::alignment::Vertical::Top,
            AlignmentSpec::Center => iced::alignment::Vertical::Center,
            AlignmentSpec::End => iced::alignment::Vertical::Bottom,
        }
    }
}
