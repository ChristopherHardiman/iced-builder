//! Property inspector sidebar.
//!
//! Displays and allows editing of properties for the selected component.

use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{Element, Length};

use crate::app::Message;
use crate::model::{
    layout::{AlignmentSpec, LengthSpec, WidgetType},
    ComponentId, LayoutNode,
};

/// Predefined color palette for text styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorChoice {
    Default,
    White,
    Black,
    Red,
    Green,
    Blue,
    Yellow,
    Orange,
    Purple,
    Gray,
}

impl ColorChoice {
    /// All available color choices.
    pub const ALL: [ColorChoice; 10] = [
        ColorChoice::Default,
        ColorChoice::White,
        ColorChoice::Black,
        ColorChoice::Red,
        ColorChoice::Green,
        ColorChoice::Blue,
        ColorChoice::Yellow,
        ColorChoice::Orange,
        ColorChoice::Purple,
        ColorChoice::Gray,
    ];

    /// Convert to RGBA array (None for default).
    pub fn to_rgba(self) -> Option<[f32; 4]> {
        match self {
            ColorChoice::Default => None,
            ColorChoice::White => Some([1.0, 1.0, 1.0, 1.0]),
            ColorChoice::Black => Some([0.0, 0.0, 0.0, 1.0]),
            ColorChoice::Red => Some([1.0, 0.2, 0.2, 1.0]),
            ColorChoice::Green => Some([0.2, 0.8, 0.2, 1.0]),
            ColorChoice::Blue => Some([0.2, 0.5, 1.0, 1.0]),
            ColorChoice::Yellow => Some([1.0, 0.9, 0.2, 1.0]),
            ColorChoice::Orange => Some([1.0, 0.6, 0.2, 1.0]),
            ColorChoice::Purple => Some([0.7, 0.3, 0.9, 1.0]),
            ColorChoice::Gray => Some([0.5, 0.5, 0.5, 1.0]),
        }
    }

    /// Create from RGBA array.
    pub fn from_rgba(color: Option<[f32; 4]>) -> Self {
        match color {
            None => ColorChoice::Default,
            Some([r, g, b, _]) => {
                // Find closest match
                if (r - 1.0).abs() < 0.1 && (g - 1.0).abs() < 0.1 && (b - 1.0).abs() < 0.1 {
                    ColorChoice::White
                } else if r < 0.1 && g < 0.1 && b < 0.1 {
                    ColorChoice::Black
                } else if r > 0.8 && g < 0.4 && b < 0.4 {
                    ColorChoice::Red
                } else if r < 0.4 && g > 0.6 && b < 0.4 {
                    ColorChoice::Green
                } else if r < 0.4 && g < 0.6 && b > 0.8 {
                    ColorChoice::Blue
                } else if r > 0.8 && g > 0.8 && b < 0.4 {
                    ColorChoice::Yellow
                } else if r > 0.8 && g > 0.4 && g < 0.8 && b < 0.4 {
                    ColorChoice::Orange
                } else if r > 0.5 && g < 0.5 && b > 0.8 {
                    ColorChoice::Purple
                } else if (r - 0.5).abs() < 0.1 && (g - 0.5).abs() < 0.1 && (b - 0.5).abs() < 0.1 {
                    ColorChoice::Gray
                } else {
                    ColorChoice::Default
                }
            }
        }
    }
}

impl std::fmt::Display for ColorChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorChoice::Default => write!(f, "Default"),
            ColorChoice::White => write!(f, "White"),
            ColorChoice::Black => write!(f, "Black"),
            ColorChoice::Red => write!(f, "Red"),
            ColorChoice::Green => write!(f, "Green"),
            ColorChoice::Blue => write!(f, "Blue"),
            ColorChoice::Yellow => write!(f, "Yellow"),
            ColorChoice::Orange => write!(f, "Orange"),
            ColorChoice::Purple => write!(f, "Purple"),
            ColorChoice::Gray => write!(f, "Gray"),
        }
    }
}

/// Length variant for the picker (simplified for UI).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthVariant {
    Fill,
    Shrink,
    Fixed,
    FillPortion,
}

impl LengthVariant {
    pub const ALL: [LengthVariant; 4] = [
        LengthVariant::Fill,
        LengthVariant::Shrink,
        LengthVariant::Fixed,
        LengthVariant::FillPortion,
    ];

    pub fn from_spec(spec: LengthSpec) -> Self {
        match spec {
            LengthSpec::Fill => LengthVariant::Fill,
            LengthSpec::Shrink => LengthVariant::Shrink,
            LengthSpec::Fixed(_) => LengthVariant::Fixed,
            LengthSpec::FillPortion(_) => LengthVariant::FillPortion,
        }
    }
}

impl std::fmt::Display for LengthVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LengthVariant::Fill => write!(f, "Fill"),
            LengthVariant::Shrink => write!(f, "Shrink"),
            LengthVariant::Fixed => write!(f, "Fixed"),
            LengthVariant::FillPortion => write!(f, "FillPortion"),
        }
    }
}

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

    /// Render container properties (padding, spacing, alignment, dimensions).
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
        
        // Get current width/height info for display
        let width_variant = LengthVariant::from_spec(attrs.width);
        let height_variant = LengthVariant::from_spec(attrs.height);
        let width_value = Self::get_length_value(attrs.width);
        let height_value = Self::get_length_value(attrs.height);
        
        // Current alignment
        let align_x = attrs.align_x;
        let align_y = attrs.align_y;
        
        column![
            Self::section_header("Layout"),
            Self::numeric_input_owned("Padding", padding_str, move |s| {
                s.parse::<f32>().ok().map(|v| Message::UpdatePadding(id, v)).unwrap_or(Message::Noop)
            }),
            Self::numeric_input_owned("Spacing", spacing_str, move |s| {
                s.parse::<f32>().ok().map(|v| Message::UpdateSpacing(id, v)).unwrap_or(Message::Noop)
            }),
            Self::section_header("Dimensions"),
            Self::length_picker("Width", id, width_variant, width_value, true),
            Self::length_picker("Height", id, height_variant, height_value, false),
            Self::section_header("Alignment"),
            Self::alignment_picker("Align X", id, align_x, true),
            Self::alignment_picker("Align Y", id, align_y, false),
            Self::section_header("Content"),
            Self::property_row_owned("Children", children_text),
        ]
        .spacing(8)
        .into()
    }

    /// Get the numeric value from a LengthSpec (for Fixed and FillPortion).
    fn get_length_value(spec: LengthSpec) -> Option<f32> {
        match spec {
            LengthSpec::Fixed(v) => Some(v),
            LengthSpec::FillPortion(v) => Some(v as f32),
            _ => None,
        }
    }

    /// Render a length picker with variant selector and optional value input.
    fn length_picker(
        label: &'static str,
        id: ComponentId,
        current_variant: LengthVariant,
        current_value: Option<f32>,
        is_width: bool,
    ) -> Column<'static, Message> {
        let variant_buttons = row![
            Self::length_button("Fill", LengthVariant::Fill, current_variant, id, is_width, None),
            Self::length_button("Shrink", LengthVariant::Shrink, current_variant, id, is_width, None),
            Self::length_button("Fixed", LengthVariant::Fixed, current_variant, id, is_width, Some(100.0)),
            Self::length_button("Portion", LengthVariant::FillPortion, current_variant, id, is_width, Some(1.0)),
        ]
        .spacing(2);

        // Show value input for Fixed and FillPortion
        let value_input: Element<'static, Message> = match current_variant {
            LengthVariant::Fixed => {
                let val_str = current_value.map(|v| format!("{}", v)).unwrap_or_default();
                text_input("100", &val_str)
                    .on_input(move |s| {
                        s.parse::<f32>().ok()
                            .map(|v| {
                                if is_width {
                                    Message::UpdateWidth(id, LengthSpec::Fixed(v))
                                } else {
                                    Message::UpdateHeight(id, LengthSpec::Fixed(v))
                                }
                            })
                            .unwrap_or(Message::Noop)
                    })
                    .size(12)
                    .width(Length::Fixed(60.0))
                    .into()
            }
            LengthVariant::FillPortion => {
                let val_str = current_value.map(|v| format!("{}", v as u16)).unwrap_or_default();
                text_input("1", &val_str)
                    .on_input(move |s| {
                        s.parse::<u16>().ok()
                            .map(|v| {
                                if is_width {
                                    Message::UpdateWidth(id, LengthSpec::FillPortion(v))
                                } else {
                                    Message::UpdateHeight(id, LengthSpec::FillPortion(v))
                                }
                            })
                            .unwrap_or(Message::Noop)
                    })
                    .size(12)
                    .width(Length::Fixed(40.0))
                    .into()
            }
            _ => text("").into(),
        };

        column![
            text(label).size(11).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            row![variant_buttons, value_input].spacing(4),
        ]
        .spacing(2)
    }

    /// Create a button for selecting a length variant.
    fn length_button(
        label: &'static str,
        variant: LengthVariant,
        current: LengthVariant,
        id: ComponentId,
        is_width: bool,
        default_value: Option<f32>,
    ) -> Element<'static, Message> {
        let is_selected = variant == current;
        let bg_color = if is_selected {
            iced::Color::from_rgb(0.2, 0.5, 0.8)
        } else {
            iced::Color::from_rgb(0.3, 0.3, 0.3)
        };
        
        let spec = match variant {
            LengthVariant::Fill => LengthSpec::Fill,
            LengthVariant::Shrink => LengthSpec::Shrink,
            LengthVariant::Fixed => LengthSpec::Fixed(default_value.unwrap_or(100.0)),
            LengthVariant::FillPortion => LengthSpec::FillPortion(default_value.unwrap_or(1.0) as u16),
        };
        
        let msg = if is_width {
            Message::UpdateWidth(id, spec)
        } else {
            Message::UpdateHeight(id, spec)
        };
        
        button(text(label).size(10))
            .on_press(msg)
            .padding(3)
            .style(move |_theme, _status| button::Style {
                background: Some(iced::Background::Color(bg_color)),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    /// Render an alignment picker.
    fn alignment_picker(
        label: &'static str,
        id: ComponentId,
        current: AlignmentSpec,
        is_x: bool,
    ) -> Column<'static, Message> {
        let buttons = row![
            Self::alignment_button("Start", AlignmentSpec::Start, current, id, is_x),
            Self::alignment_button("Center", AlignmentSpec::Center, current, id, is_x),
            Self::alignment_button("End", AlignmentSpec::End, current, id, is_x),
        ]
        .spacing(2);

        column![
            text(label).size(11).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            buttons,
        ]
        .spacing(2)
    }

    /// Create a button for selecting an alignment.
    fn alignment_button(
        label: &'static str,
        alignment: AlignmentSpec,
        current: AlignmentSpec,
        id: ComponentId,
        is_x: bool,
    ) -> Element<'static, Message> {
        let is_selected = alignment == current;
        let bg_color = if is_selected {
            iced::Color::from_rgb(0.2, 0.5, 0.8)
        } else {
            iced::Color::from_rgb(0.3, 0.3, 0.3)
        };
        
        let msg = if is_x {
            Message::UpdateAlignX(id, alignment)
        } else {
            Message::UpdateAlignY(id, alignment)
        };
        
        button(text(label).size(10))
            .on_press(msg)
            .padding(3)
            .style(move |_theme, _status| button::Style {
                background: Some(iced::Background::Color(bg_color)),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }

    /// Render text properties.
    fn render_text_props(
        id: ComponentId,
        content: &str,
        attrs: &crate::model::layout::TextAttrs,
    ) -> Element<'static, Message> {
        let font_size_str = format!("{}", attrs.font_size);
        let current_color = ColorChoice::from_rgba(attrs.color);
        let content_owned = content.to_string();
        
        column![
            Self::section_header("Content"),
            Self::labeled_input_owned("Text", content_owned, move |s| Message::UpdateTextContent(id, s)),
            Self::section_header("Style"),
            Self::numeric_input_owned("Font Size", font_size_str, move |s| {
                s.parse::<f32>().ok().map(|v| Message::UpdateFontSize(id, v)).unwrap_or(Message::Noop)
            }),
            Self::property_row_static("Alignment", Self::alignment_display(attrs.horizontal_alignment)),
            Self::color_picker("Color", id, current_color),
        ]
        .spacing(8)
        .into()
    }

    /// Render a color picker.
    fn color_picker(
        label: &'static str,
        id: ComponentId,
        current: ColorChoice,
    ) -> Column<'static, Message> {
        let buttons = row![
            Self::color_button(ColorChoice::Default, current, id),
            Self::color_button(ColorChoice::White, current, id),
            Self::color_button(ColorChoice::Black, current, id),
            Self::color_button(ColorChoice::Red, current, id),
            Self::color_button(ColorChoice::Green, current, id),
        ]
        .spacing(2);

        let buttons2 = row![
            Self::color_button(ColorChoice::Blue, current, id),
            Self::color_button(ColorChoice::Yellow, current, id),
            Self::color_button(ColorChoice::Orange, current, id),
            Self::color_button(ColorChoice::Purple, current, id),
            Self::color_button(ColorChoice::Gray, current, id),
        ]
        .spacing(2);

        column![
            text(label).size(11).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            buttons,
            buttons2,
        ]
        .spacing(2)
    }

    /// Create a button for selecting a color.
    fn color_button(
        color: ColorChoice,
        current: ColorChoice,
        id: ComponentId,
    ) -> Element<'static, Message> {
        let is_selected = color == current;
        let rgba = color.to_rgba().unwrap_or([0.3, 0.3, 0.3, 1.0]);
        let bg = iced::Color::from_rgba(rgba[0], rgba[1], rgba[2], rgba[3]);
        
        // For default, show a special indicator
        let display_color = if matches!(color, ColorChoice::Default) {
            iced::Color::from_rgb(0.4, 0.4, 0.4)
        } else {
            bg
        };
        
        let border_color = if is_selected {
            iced::Color::from_rgb(0.2, 0.6, 1.0)
        } else {
            iced::Color::from_rgb(0.2, 0.2, 0.2)
        };
        
        let label_text = if matches!(color, ColorChoice::Default) {
            "Def"
        } else {
            ""
        };
        
        button(text(label_text).size(8))
            .on_press(Message::UpdateTextColor(id, color.to_rgba()))
            .padding(2)
            .width(Length::Fixed(22.0))
            .height(Length::Fixed(22.0))
            .style(move |_theme, _status| button::Style {
                background: Some(iced::Background::Color(display_color)),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    color: border_color,
                    width: if is_selected { 2.0 } else { 1.0 },
                    radius: 3.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    /// Labeled input with owned value.
    fn labeled_input_owned<F>(label: &'static str, value: String, on_change: F) -> Column<'static, Message>
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
