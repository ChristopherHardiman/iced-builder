//! Main application state and logic.
//!
//! Contains the top-level App struct, Message enum, and update/view functions.

use iced::widget::{column, container, horizontal_rule, row, text, vertical_rule};
use iced::{Element, Length, Subscription, Task};

use crate::model::{ComponentId, LayoutNode, Project, ProjectConfig};
use crate::ui::{palette::WidgetKind, Canvas, Inspector, Palette, TreeView};

/// Editor mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorMode {
    #[default]
    Design,
    Preview,
}

/// Application state.
#[derive(Debug)]
pub struct App {
    /// The currently open project.
    project: Option<Project>,
    /// Current editor mode.
    mode: EditorMode,
    /// Status message to display.
    status_message: Option<String>,
}

/// Messages for the application.
#[derive(Debug, Clone)]
pub enum Message {
    // File operations
    NewProject,
    OpenProject,
    SaveProject,
    ExportCode,
    ProjectOpened(Result<Project, String>),

    // Selection
    SelectComponent(ComponentId),
    DeselectComponent,

    // Palette
    PaletteItemClicked(WidgetKind),

    // Component operations
    DeleteSelected,

    // Undo/Redo
    Undo,
    Redo,

    // Mode
    SetMode(EditorMode),

    // Property updates
    UpdateTextContent(ComponentId, String),
    UpdateButtonLabel(ComponentId, String),
    UpdateMessageStub(ComponentId, String),
    UpdatePlaceholder(ComponentId, String),
    UpdateBinding(ComponentId, String),

    // No-op (for disabled widgets)
    Noop,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Create a new application instance.
    pub fn new() -> Self {
        Self {
            project: None,
            mode: EditorMode::Design,
            status_message: None,
        }
    }

    /// Get the window title.
    pub fn title(&self) -> String {
        match &self.project {
            Some(p) => {
                let dirty = if p.dirty { " •" } else { "" };
                format!("Iced Builder - {}{}", p.layout.name, dirty)
            }
            None => String::from("Iced Builder"),
        }
    }

    /// Update application state based on a message.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NewProject => {
                // Create a new project with default layout
                let config = ProjectConfig::default();
                let project = Project::new(std::path::PathBuf::from("."), config);
                self.project = Some(project);
                self.status_message = Some("New project created".to_string());
                Task::none()
            }

            Message::OpenProject => {
                // TODO: Open file dialog
                self.status_message = Some("Open project not yet implemented".to_string());
                Task::none()
            }

            Message::SaveProject => {
                if let Some(project) = &mut self.project {
                    project.mark_saved();
                    self.status_message = Some("Project saved".to_string());
                }
                Task::none()
            }

            Message::ExportCode => {
                if let Some(project) = &self.project {
                    let code = crate::codegen::generate_code(&project.layout, &project.config);
                    let formatted = crate::util::try_format_rust_code(&code);
                    // For now, just print to console
                    println!("Generated code:\n{}", formatted);
                    self.status_message = Some("Code exported (see console)".to_string());
                }
                Task::none()
            }

            Message::ProjectOpened(result) => {
                match result {
                    Ok(project) => {
                        self.project = Some(project);
                        self.status_message = Some("Project opened".to_string());
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Failed to open project: {}", e));
                    }
                }
                Task::none()
            }

            Message::SelectComponent(id) => {
                if let Some(project) = &mut self.project {
                    project.selected_id = Some(id);
                }
                Task::none()
            }

            Message::DeselectComponent => {
                if let Some(project) = &mut self.project {
                    project.selected_id = None;
                }
                Task::none()
            }

            Message::PaletteItemClicked(kind) => {
                if let Some(project) = &mut self.project {
                    // Push history before modification
                    project.history.push(project.layout.clone());

                    // Create the new node
                    let new_node = create_node_for_kind(kind);

                    // Add to root (or selected container)
                    if let Some(children) = project.layout.root.children_mut() {
                        children.push(new_node);
                    }

                    project.rebuild_index();
                    project.mark_dirty();
                    self.status_message = Some(format!("Added {}", kind.name()));
                }
                Task::none()
            }

            Message::DeleteSelected => {
                if let Some(project) = &mut self.project {
                    if project.selected_id.is_some() {
                        // TODO: Actually delete the selected component
                        project.selected_id = None;
                        self.status_message = Some("Delete not yet implemented".to_string());
                    }
                }
                Task::none()
            }

            Message::Undo => {
                if let Some(project) = &mut self.project {
                    if let Some(previous) = project.history.undo(project.layout.clone()) {
                        project.layout = previous;
                        project.rebuild_index();
                        self.status_message = Some("Undo".to_string());
                    }
                }
                Task::none()
            }

            Message::Redo => {
                if let Some(project) = &mut self.project {
                    if let Some(next) = project.history.redo(project.layout.clone()) {
                        project.layout = next;
                        project.rebuild_index();
                        self.status_message = Some("Redo".to_string());
                    }
                }
                Task::none()
            }

            Message::SetMode(mode) => {
                self.mode = mode;
                Task::none()
            }

            Message::UpdateTextContent(id, content) => {
                self.update_node_property(id, |node| {
                    if let crate::model::layout::WidgetType::Text { content: c, .. } = &mut node.widget {
                        *c = content;
                    }
                });
                Task::none()
            }

            Message::UpdateButtonLabel(id, label) => {
                self.update_node_property(id, |node| {
                    if let crate::model::layout::WidgetType::Button { label: l, .. } = &mut node.widget {
                        *l = label;
                    }
                });
                Task::none()
            }

            Message::UpdateMessageStub(id, stub) => {
                self.update_node_property(id, |node| {
                    match &mut node.widget {
                        crate::model::layout::WidgetType::Button { message_stub, .. } => *message_stub = stub,
                        crate::model::layout::WidgetType::TextInput { message_stub, .. } => *message_stub = stub,
                        crate::model::layout::WidgetType::Checkbox { message_stub, .. } => *message_stub = stub,
                        crate::model::layout::WidgetType::Slider { message_stub, .. } => *message_stub = stub,
                        crate::model::layout::WidgetType::PickList { message_stub, .. } => *message_stub = stub,
                        _ => {}
                    }
                });
                Task::none()
            }

            Message::UpdatePlaceholder(id, placeholder) => {
                self.update_node_property(id, |node| {
                    if let crate::model::layout::WidgetType::TextInput { placeholder: p, .. } = &mut node.widget {
                        *p = placeholder;
                    }
                });
                Task::none()
            }

            Message::UpdateBinding(id, binding) => {
                self.update_node_property(id, |node| {
                    match &mut node.widget {
                        crate::model::layout::WidgetType::TextInput { value_binding, .. } => *value_binding = binding,
                        crate::model::layout::WidgetType::Checkbox { checked_binding, .. } => *checked_binding = binding,
                        crate::model::layout::WidgetType::Slider { value_binding, .. } => *value_binding = binding,
                        crate::model::layout::WidgetType::PickList { selected_binding, .. } => *selected_binding = binding,
                        _ => {}
                    }
                });
                Task::none()
            }

            Message::Noop => Task::none(),
        }
    }

    /// Helper to update a node property.
    fn update_node_property<F>(&mut self, _id: ComponentId, _update_fn: F)
    where
        F: FnOnce(&mut LayoutNode),
    {
        // TODO: Find node by ID and apply update
        if let Some(project) = &mut self.project {
            project.mark_dirty();
        }
    }

    /// Render the application view.
    pub fn view(&self) -> Element<'_, Message> {
        let palette = Palette::view();

        let canvas: Element<Message> = match &self.project {
            Some(project) => Canvas::view(&project.layout.root, project.selected_id),
            None => Canvas::view_empty(),
        };

        let inspector: Element<Message> = match &self.project {
            Some(project) => {
                let selected_node = project
                    .selected_id
                    .and_then(|id| project.find_node(id));
                Inspector::view(selected_node, project.selected_id)
            }
            None => Inspector::view(None, None),
        };

        let tree_view: Element<Message> = match &self.project {
            Some(project) => TreeView::view(&project.layout.root, project.selected_id),
            None => container(text("No project")).into(),
        };

        // Status bar
        let status = container(
            text(self.status_message.as_deref().unwrap_or("Ready"))
                .size(12)
                .color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
        )
        .padding(5);

        // Main layout: palette | canvas | inspector
        let main_row = row![
            palette,
            vertical_rule(1),
            column![canvas, horizontal_rule(1), tree_view].width(Length::Fill),
            vertical_rule(1),
            inspector,
        ]
        .height(Length::Fill);

        // Full layout with status bar
        column![main_row, horizontal_rule(1), status].into()
    }

    /// Handle subscriptions (keyboard shortcuts).
    pub fn subscription(&self) -> Subscription<Message> {
        use iced::keyboard;

        keyboard::on_key_press(|key, modifiers| {
            match (key.as_ref(), modifiers.command(), modifiers.shift()) {
                (keyboard::Key::Character("z"), true, false) => Some(Message::Undo),
                (keyboard::Key::Character("z"), true, true) => Some(Message::Redo),
                (keyboard::Key::Character("y"), true, false) => Some(Message::Redo),
                (keyboard::Key::Character("s"), true, false) => Some(Message::SaveProject),
                (keyboard::Key::Character("e"), true, false) => Some(Message::ExportCode),
                (keyboard::Key::Character("n"), true, false) => Some(Message::NewProject),
                (keyboard::Key::Named(keyboard::key::Named::Delete), false, false) => {
                    Some(Message::DeleteSelected)
                }
                (keyboard::Key::Named(keyboard::key::Named::Escape), false, false) => {
                    Some(Message::DeselectComponent)
                }
                _ => None,
            }
        })
    }
}

/// Create a new LayoutNode for the given widget kind.
fn create_node_for_kind(kind: WidgetKind) -> LayoutNode {
    use crate::model::layout::*;

    let widget = match kind {
        WidgetKind::ColumnContainer => WidgetType::Column {
            children: Vec::new(),
            attrs: ContainerAttrs::default(),
        },
        WidgetKind::RowContainer => WidgetType::Row {
            children: Vec::new(),
            attrs: ContainerAttrs::default(),
        },
        WidgetKind::Container => WidgetType::Container {
            child: None,
            attrs: ContainerAttrs::default(),
        },
        WidgetKind::Scrollable => WidgetType::Scrollable {
            child: None,
            attrs: ContainerAttrs::default(),
        },
        WidgetKind::Stack => WidgetType::Stack {
            children: Vec::new(),
            attrs: ContainerAttrs::default(),
        },
        WidgetKind::Text => WidgetType::Text {
            content: String::from("Text"),
            attrs: TextAttrs::default(),
        },
        WidgetKind::Button => WidgetType::Button {
            label: String::from("Button"),
            message_stub: String::from("ButtonPressed"),
            attrs: ButtonAttrs::default(),
        },
        WidgetKind::TextInput => WidgetType::TextInput {
            placeholder: String::from("Enter text..."),
            value_binding: String::from("input_value"),
            message_stub: String::from("InputChanged"),
            attrs: InputAttrs::default(),
        },
        WidgetKind::Checkbox => WidgetType::Checkbox {
            label: String::from("Checkbox"),
            checked_binding: String::from("is_checked"),
            message_stub: String::from("CheckboxToggled"),
            attrs: CheckboxAttrs::default(),
        },
        WidgetKind::Slider => WidgetType::Slider {
            min: 0.0,
            max: 100.0,
            value_binding: String::from("slider_value"),
            message_stub: String::from("SliderChanged"),
            attrs: SliderAttrs::default(),
        },
        WidgetKind::PickList => WidgetType::PickList {
            options: vec![String::from("Option 1"), String::from("Option 2")],
            selected_binding: String::from("selected_option"),
            message_stub: String::from("OptionSelected"),
            attrs: PickListAttrs::default(),
        },
        WidgetKind::Space => WidgetType::Space {
            width: LengthSpec::Fixed(20.0),
            height: LengthSpec::Fixed(20.0),
        },
    };

    LayoutNode::new(widget)
}
