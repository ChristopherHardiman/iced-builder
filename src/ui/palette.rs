//! Widget palette sidebar.
//!
//! Displays available widgets and containers that can be added to the layout.

use iced::widget::{button, column, container, scrollable, text, Column};
use iced::{Element, Length};

use crate::app::Message;

/// Widget categories in the palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetCategory {
    Containers,
    Widgets,
}

/// Types of widgets that can be added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetKind {
    // Containers
    ColumnContainer,
    RowContainer,
    Container,
    Scrollable,
    Stack,
    // Widgets
    Text,
    Button,
    TextInput,
    Checkbox,
    Slider,
    PickList,
    Space,
}

impl WidgetKind {
    /// Get the display name for this widget type.
    pub fn name(&self) -> &'static str {
        match self {
            Self::ColumnContainer => "Column",
            Self::RowContainer => "Row",
            Self::Container => "Container",
            Self::Scrollable => "Scrollable",
            Self::Stack => "Stack",
            Self::Text => "Text",
            Self::Button => "Button",
            Self::TextInput => "TextInput",
            Self::Checkbox => "Checkbox",
            Self::Slider => "Slider",
            Self::PickList => "PickList",
            Self::Space => "Space",
        }
    }

    /// Get the category for this widget type.
    pub fn category(&self) -> WidgetCategory {
        match self {
            Self::ColumnContainer
            | Self::RowContainer
            | Self::Container
            | Self::Scrollable
            | Self::Stack => WidgetCategory::Containers,
            _ => WidgetCategory::Widgets,
        }
    }

    /// Get all container widget kinds.
    pub fn containers() -> &'static [WidgetKind] {
        &[
            Self::ColumnContainer,
            Self::RowContainer,
            Self::Container,
            Self::Scrollable,
            Self::Stack,
        ]
    }

    /// Get all basic widget kinds.
    pub fn widgets() -> &'static [WidgetKind] {
        &[
            Self::Text,
            Self::Button,
            Self::TextInput,
            Self::Checkbox,
            Self::Slider,
            Self::PickList,
            Self::Space,
        ]
    }
}

/// The widget palette component.
pub struct Palette;

impl Palette {
    /// Render the palette sidebar.
    pub fn view<'a>() -> Element<'a, Message> {
        let container_section = Self::section("Containers", WidgetKind::containers());
        let widget_section = Self::section("Widgets", WidgetKind::widgets());

        let content = column![container_section, widget_section]
            .spacing(20)
            .padding(10)
            .width(Length::Fill);

        container(scrollable(content).height(Length::Fill))
            .width(Length::Fixed(180.0))
            .height(Length::Fill)
            .into()
    }

    /// Render a section of the palette.
    fn section<'a>(title: &'a str, kinds: &[WidgetKind]) -> Column<'a, Message> {
        let header = text(title).size(14);

        let buttons: Vec<Element<'a, Message>> = kinds
            .iter()
            .map(|kind| {
                button(text(kind.name()).size(13))
                    .on_press(Message::PaletteItemClicked(*kind))
                    .width(Length::Fill)
                    .into()
            })
            .collect();

        let mut col = column![header].spacing(5);
        for btn in buttons {
            col = col.push(btn);
        }
        col
    }
}
