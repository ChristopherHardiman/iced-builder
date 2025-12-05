//! Main application state and logic.
//!
//! Contains the top-level App struct, Message enum, and update/view functions.

use iced::widget::{button, column, container, horizontal_rule, row, text, vertical_rule};
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
    CreateProjectAt(std::path::PathBuf),
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
    
    // Container property updates
    UpdatePadding(ComponentId, f32),
    UpdateSpacing(ComponentId, f32),
    
    // Checkbox property updates
    UpdateCheckboxLabel(ComponentId, String),
    
    // Slider property updates
    UpdateSliderRange(ComponentId, f32, f32),

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
        tracing::debug!(target: "iced_builder::app::message", ?message, "Processing message");
        
        match message {
            Message::NewProject => {
                tracing::info!(target: "iced_builder::app", "Creating new project");
                // Open folder picker for new project location
                Task::perform(
                    async {
                        let folder = rfd::AsyncFileDialog::new()
                            .set_title("Select folder for new project")
                            .pick_folder()
                            .await;
                        folder.map(|f| f.path().to_path_buf())
                    },
                    |path| match path {
                        Some(path) => Message::CreateProjectAt(path),
                        None => Message::Noop,
                    },
                )
            }

            Message::CreateProjectAt(path) => {
                tracing::info!(target: "iced_builder::app", path = %path.display(), "Creating project at path");
                match Project::create(&path, None) {
                    Ok(project) => {
                        self.project = Some(project);
                        self.status_message = Some("New project created".to_string());
                    }
                    Err(e) => {
                        tracing::error!(target: "iced_builder::app", error = %e, "Failed to create project");
                        self.status_message = Some(format!("Failed to create project: {}", e));
                    }
                }
                Task::none()
            }

            Message::OpenProject => {
                tracing::info!(target: "iced_builder::app", "Open project requested");
                // Open folder picker dialog
                Task::perform(
                    async {
                        let folder = rfd::AsyncFileDialog::new()
                            .set_title("Open Iced Builder Project")
                            .pick_folder()
                            .await;
                        
                        match folder {
                            Some(f) => {
                                let path = f.path().to_path_buf();
                                Project::open(&path)
                                    .map_err(|e| e.to_string())
                            }
                            None => Err("No folder selected".to_string()),
                        }
                    },
                    Message::ProjectOpened,
                )
            }

            Message::SaveProject => {
                tracing::info!(target: "iced_builder::app", "Saving project");
                if let Some(project) = &mut self.project {
                    match project.save() {
                        Ok(()) => {
                            self.status_message = Some("Project saved".to_string());
                        }
                        Err(e) => {
                            tracing::error!(target: "iced_builder::app", error = %e, "Failed to save project");
                            self.status_message = Some(format!("Failed to save: {}", e));
                        }
                    }
                } else {
                    self.status_message = Some("No project open".to_string());
                }
                Task::none()
            }

            Message::ExportCode => {
                tracing::info!(target: "iced_builder::codegen", "Exporting code");
                if let Some(project) = &self.project {
                    match project.export() {
                        Ok(code) => {
                            tracing::debug!(target: "iced_builder::codegen", code_length = code.len(), "Code generated");
                            self.status_message = Some(format!(
                                "Code exported to {}",
                                project.config.output_file.display()
                            ));
                        }
                        Err(e) => {
                            tracing::error!(target: "iced_builder::codegen", error = %e, "Export failed");
                            self.status_message = Some(format!("Export failed: {}", e));
                        }
                    }
                } else {
                    self.status_message = Some("No project open".to_string());
                }
                Task::none()
            }

            Message::ProjectOpened(result) => {
                match result {
                    Ok(project) => {
                        tracing::info!(target: "iced_builder::app", name = %project.layout.name, "Project opened");
                        self.project = Some(project);
                        self.status_message = Some("Project opened".to_string());
                    }
                    Err(e) => {
                        tracing::error!(target: "iced_builder::app", error = %e, "Failed to open project");
                        // Show a shorter message in status bar
                        let short_msg = if e.to_string().contains("Not an Iced Builder project") {
                            "Not an Iced Builder project. Use 'New Project' to create one.".to_string()
                        } else {
                            format!("Failed to open: {}", e)
                        };
                        self.status_message = Some(short_msg);
                    }
                }
                Task::none()
            }

            Message::SelectComponent(id) => {
                tracing::debug!(target: "iced_builder::app::selection", %id, "Component selected");
                if let Some(project) = &mut self.project {
                    project.selected_id = Some(id);
                    
                    // Log details about the selected node
                    if let Some(node) = project.find_node(id) {
                        tracing::debug!(
                            target: "iced_builder::app::selection",
                            widget_type = ?std::mem::discriminant(&node.widget),
                            "Selected node details"
                        );
                    }
                }
                Task::none()
            }

            Message::DeselectComponent => {
                tracing::debug!(target: "iced_builder::app::selection", "Component deselected");
                if let Some(project) = &mut self.project {
                    project.selected_id = None;
                }
                Task::none()
            }

            Message::PaletteItemClicked(kind) => {
                tracing::info!(target: "iced_builder::app::tree", ?kind, "Adding widget from palette");
                if let Some(project) = &mut self.project {
                    // Push history before modification
                    project.history.push(project.layout.clone());

                    // Create the new node
                    let new_node = create_node_for_kind(kind);
                    let new_node_id = new_node.id;
                    tracing::debug!(
                        target: "iced_builder::app::tree", 
                        node_id = %new_node.id, 
                        "Created new node"
                    );

                    // Try to add to selected container, otherwise add to root
                    let added = if let Some(selected_id) = project.selected_id {
                        if project.is_container(selected_id) {
                            tracing::debug!(
                                target: "iced_builder::app::tree",
                                parent_id = %selected_id,
                                "Adding to selected container"
                            );
                            project.add_child_to_node(selected_id, new_node)
                        } else {
                            tracing::debug!(
                                target: "iced_builder::app::tree",
                                "Selected node is not a container, adding to root"
                            );
                            project.add_child_to_root(new_node)
                        }
                    } else {
                        tracing::debug!(
                            target: "iced_builder::app::tree",
                            "No selection, adding to root"
                        );
                        project.add_child_to_root(new_node)
                    };

                    if added {
                        project.mark_dirty();
                        // Select the newly added node
                        project.selected_id = Some(new_node_id);
                        self.status_message = Some(format!("Added {}", kind.name()));
                    } else {
                        // Undo the history push if add failed
                        let _ = project.history.undo(project.layout.clone());
                        self.status_message = Some("Cannot add widget here".to_string());
                    }
                }
                Task::none()
            }

            Message::DeleteSelected => {
                if let Some(project) = &mut self.project {
                    if let Some(id) = project.selected_id {
                        tracing::info!(target: "iced_builder::app::tree", %id, "Delete requested");
                        
                        // Push history before modification
                        project.history.push(project.layout.clone());
                        
                        // Remove the selected node
                        if project.remove_node(id) {
                            project.selected_id = None;
                            project.mark_dirty();
                            tracing::info!(target: "iced_builder::app::tree", %id, "Component deleted");
                            self.status_message = Some("Component deleted".to_string());
                        } else {
                            // Undo the history push if removal failed
                            let _ = project.history.undo(project.layout.clone());
                            tracing::warn!(target: "iced_builder::app::tree", %id, "Failed to delete component");
                            self.status_message = Some("Cannot delete this component".to_string());
                        }
                    }
                }
                Task::none()
            }

            Message::Undo => {
                tracing::debug!(target: "iced_builder::app", "Undo requested");
                if let Some(project) = &mut self.project {
                    if let Some(previous) = project.history.undo(project.layout.clone()) {
                        project.layout = previous;
                        project.rebuild_index();
                        tracing::info!(target: "iced_builder::app", "Undo applied");
                        self.status_message = Some("Undo".to_string());
                    }
                }
                Task::none()
            }

            Message::Redo => {
                tracing::debug!(target: "iced_builder::app", "Redo requested");
                if let Some(project) = &mut self.project {
                    if let Some(next) = project.history.redo(project.layout.clone()) {
                        project.layout = next;
                        project.rebuild_index();
                        tracing::info!(target: "iced_builder::app", "Redo applied");
                        self.status_message = Some("Redo".to_string());
                    }
                }
                Task::none()
            }

            Message::SetMode(mode) => {
                tracing::debug!(target: "iced_builder::app", ?mode, "Mode changed");
                self.mode = mode;
                Task::none()
            }

            Message::UpdateTextContent(id, content) => {
                tracing::debug!(target: "iced_builder::ui::inspector", %id, "Updating text content");
                self.update_node_property(id, |node| {
                    if let crate::model::layout::WidgetType::Text { content: c, .. } = &mut node.widget {
                        *c = content;
                    }
                });
                Task::none()
            }

            Message::UpdateButtonLabel(id, label) => {
                tracing::debug!(target: "iced_builder::ui::inspector", %id, "Updating button label");
                self.update_node_property(id, |node| {
                    if let crate::model::layout::WidgetType::Button { label: l, .. } = &mut node.widget {
                        *l = label;
                    }
                });
                Task::none()
            }

            Message::UpdateMessageStub(id, stub) => {
                tracing::debug!(target: "iced_builder::ui::inspector", %id, "Updating message stub");
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

            Message::UpdatePadding(id, padding) => {
                self.update_node_property(id, |node| {
                    match &mut node.widget {
                        crate::model::layout::WidgetType::Column { attrs, .. }
                        | crate::model::layout::WidgetType::Row { attrs, .. }
                        | crate::model::layout::WidgetType::Container { attrs, .. }
                        | crate::model::layout::WidgetType::Scrollable { attrs, .. }
                        | crate::model::layout::WidgetType::Stack { attrs, .. } => {
                            attrs.padding = crate::model::layout::PaddingSpec {
                                top: padding,
                                right: padding,
                                bottom: padding,
                                left: padding,
                            };
                        }
                        _ => {}
                    }
                });
                Task::none()
            }

            Message::UpdateSpacing(id, spacing) => {
                self.update_node_property(id, |node| {
                    match &mut node.widget {
                        crate::model::layout::WidgetType::Column { attrs, .. }
                        | crate::model::layout::WidgetType::Row { attrs, .. }
                        | crate::model::layout::WidgetType::Container { attrs, .. }
                        | crate::model::layout::WidgetType::Scrollable { attrs, .. }
                        | crate::model::layout::WidgetType::Stack { attrs, .. } => {
                            attrs.spacing = spacing;
                        }
                        _ => {}
                    }
                });
                Task::none()
            }

            Message::UpdateCheckboxLabel(id, label) => {
                self.update_node_property(id, |node| {
                    if let crate::model::layout::WidgetType::Checkbox { label: l, .. } = &mut node.widget {
                        *l = label;
                    }
                });
                Task::none()
            }

            Message::UpdateSliderRange(id, min, max) => {
                self.update_node_property(id, |node| {
                    if let crate::model::layout::WidgetType::Slider { min: m, max: mx, .. } = &mut node.widget {
                        *m = min;
                        *mx = max;
                    }
                });
                Task::none()
            }

            Message::Noop => Task::none(),
        }
    }

    /// Helper to update a node property with history tracking.
    fn update_node_property<F>(&mut self, id: ComponentId, update_fn: F)
    where
        F: FnOnce(&mut LayoutNode),
    {
        if let Some(project) = &mut self.project {
            // Push history before modification
            project.history.push(project.layout.clone());
            
            // Find and update the node
            if let Some(node) = project.find_node_mut(id) {
                update_fn(node);
                tracing::debug!(target: "iced_builder::app::property", %id, "Property updated");
                project.mark_dirty();
            } else {
                // Undo the history push if node not found
                let _ = project.history.undo(project.layout.clone());
                tracing::warn!(target: "iced_builder::app::property", %id, "Node not found for property update");
            }
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

        // Build status bar content
        let status_text = self.status_message.as_deref().unwrap_or("Ready");
        let history_status = match &self.project {
            Some(project) => {
                let can_undo = project.history.can_undo();
                let can_redo = project.history.can_redo();
                format!(
                    " | Undo: {} | Redo: {}",
                    if can_undo { "Ctrl+Z" } else { "-" },
                    if can_redo { "Ctrl+Y" } else { "-" }
                )
            }
            None => String::new(),
        };
        
        let dirty_indicator = match &self.project {
            Some(project) if project.dirty => " [unsaved]",
            _ => "",
        };

        // Toolbar with file operations
        let toolbar = container(
            row![
                button(text("New Project").size(12))
                    .on_press(Message::NewProject)
                    .padding([4, 8]),
                button(text("Open Project").size(12))
                    .on_press(Message::OpenProject)
                    .padding([4, 8]),
                button(text("Save").size(12))
                    .on_press(Message::SaveProject)
                    .padding([4, 8]),
                button(text("Export Code").size(12))
                    .on_press(Message::ExportCode)
                    .padding([4, 8]),
            ]
            .spacing(5),
        )
        .padding(5)
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.2, 0.2))),
            ..Default::default()
        });

        // Status bar
        let status = container(
            text(format!("{}{}{}", status_text, dirty_indicator, history_status))
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

        // Full layout with toolbar, main content, and status bar
        column![toolbar, horizontal_rule(1), main_row, horizontal_rule(1), status].into()
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
