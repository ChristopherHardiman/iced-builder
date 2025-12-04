# Iced Builder – Implementation Plan

This document outlines a phased, step-by-step plan for implementing the Iced Builder application as described in `scope.md`. It focuses on architecture decisions, module responsibilities, data flow, and implementation order—without embedding code.

---

## Phase 0: Project Setup & Tooling

### 0.1 Cargo Configuration
- Set the Rust edition to `2021`.
- Add dependencies:
  - `iced = "0.13"` (pin minor version to avoid breaking changes).
  - `serde = { version = "1", features = ["derive"] }`, `serde_json`, `ron` for serialization.
  - `toml` for parsing `iced_builder.toml`.
  - `uuid = { version = "1", features = ["v4", "serde"] }` for generating unique component IDs.
  - `rfd = "0.15"` for native file dialogs (XDG Portal on Linux, native on Windows/macOS).
  - `thiserror` for custom error types.
  - `anyhow` (optional) for prototyping error propagation.
  - `regex` for validating Rust identifier patterns.
- Add a `[profile.release]` section with LTO and size optimizations for portable binaries.
- Add `#![windows_subsystem = "windows"]` to `main.rs` for Windows builds.

### 0.2 Directory Structure
Organize the source tree for clarity and separation of concerns:

```
src/
├── main.rs              # Entry point: Iced application bootstrap
├── app.rs               # Top-level App struct, Message enum, update/view
├── model/
│   ├── mod.rs
│   ├── layout.rs        # Layout AST types (LayoutNode, Attributes, etc.)
│   ├── project.rs       # Project config and state (paths, settings)
│   └── history.rs       # Undo/redo state management
├── ui/
│   ├── mod.rs
│   ├── palette.rs       # Widget palette sidebar
│   ├── canvas.rs        # Center canvas/viewport rendering
│   ├── inspector.rs     # Property inspector sidebar
│   └── tree_view.rs     # Hierarchical tree view of layout
├── codegen/
│   ├── mod.rs
│   └── generator.rs     # Layout AST → Rust code generation
├── io/
│   ├── mod.rs
│   ├── layout_file.rs   # Load/save layout files (JSON/RON)
│   └── config.rs        # Parse iced_builder.toml
└── util.rs              # Shared helpers (ID generation, formatting invocation)
```

### 0.3 Development Workflow
- Use `cargo watch -x run` for rapid iteration.
- Keep a sample project folder (e.g., `examples/sample_project/`) with an `iced_builder.toml` and a `layout.ron` for testing.

### 0.4 Early Prototype: Canvas Interaction (Risk Mitigation)
Before proceeding with full implementation, build a minimal prototype to validate:
- Wrapping Iced widgets in `MouseArea` for click interception.
- Nested containers properly bubble selection events.
- `Container::style` can apply selection borders without layout issues.
- TextInput focus can be intercepted or disabled in design mode.

This de-risks Phase 6 (Canvas) by confirming the core UX assumption early.

---

## Phase 1: Data Model ("Layout AST")

### 1.1 Define Core Types

#### LayoutNode
- An enum representing every supported widget and container.
- Variants:
  - `Column { children: Vec<LayoutNode>, attrs: ContainerAttrs }`
  - `Row { children: Vec<LayoutNode>, attrs: ContainerAttrs }`
  - `Container { child: Box<LayoutNode>, attrs: ContainerAttrs }`
  - `Scrollable { child: Box<LayoutNode>, attrs: ContainerAttrs }`
  - `Text { content: String, attrs: TextAttrs }`
  - `Button { label: String, message_stub: String, attrs: ButtonAttrs }`
  - `TextInput { placeholder: String, value_binding: String, attrs: InputAttrs }`
  - `Checkbox { label: String, checked_binding: String, attrs: CheckboxAttrs }`
  - `Slider { min: f32, max: f32, value_binding: String, attrs: SliderAttrs }`
  - `PickList { options: Vec<String>, selected_binding: String, attrs: PickListAttrs }`

#### Attributes Structs
- `ContainerAttrs`: padding, spacing, align_items, width, height.
- `TextAttrs`: font_size, color, horizontal_alignment.
- `ButtonAttrs`: width, height, style preset.
- `InputAttrs`, `CheckboxAttrs`, `SliderAttrs`, `PickListAttrs`: similar pattern.

#### ComponentId
- Use `uuid::Uuid` for unique, stable identification of each node.
- Wrap in a newtype for type safety: `struct ComponentId(Uuid)` with `Serialize`/`Deserialize`.
- Implement `Display` for debugging and tree view labels.

#### NodeIndex
- Maintain a `HashMap<ComponentId, Vec<usize>>` mapping IDs to tree paths for O(1) lookup.
- Rebuild index on layout load or structural changes.
- Alternative: use `id_tree` or `indextree` crate for built-in traversal support.

#### Default Attribute Values
Define explicit defaults for all attributes (used when creating new nodes):
- `padding: Padding::ZERO`
- `spacing: 0.0`
- `width: Length::Shrink`
- `height: Length::Shrink`
- `align_items: Alignment::Start`
- `font_size: 16` (pixels)

#### LayoutDocument
- Top-level struct holding the root `LayoutNode` and any document-level metadata (e.g., name, version).

### 1.2 Serialization
- Derive `Serialize` and `Deserialize` for all model types.
- Prefer RON for human-readable layout files; support JSON as an alternative.
- Write unit tests that round-trip a sample layout through serialization.

### 1.3 Validation
- Implement a `validate(&self) -> Vec<ValidationError>` method on `LayoutNode`.
- Check constraints:
  - Leaf nodes (Text, Button, etc.) must not have children fields populated.
  - Bindings and message stubs must be valid Rust identifiers: `^[a-zA-Z_][a-zA-Z0-9_]*$`.
  - Empty containers (Column/Row with zero children) are valid but emit a warning.
  - Move operations must validate target is not a descendant of the source (prevents circular references).
- Return a list of warnings/errors with paths to the offending nodes (e.g., `"root.children[2].child"`).

---

## Phase 2: Project & Configuration

### 2.1 Config File (`iced_builder.toml`)
Define the expected structure:
- `project_root`: String path (optional, defaults to config file directory).
- `output_file`: Relative path for generated Rust code.
- `message_type`: Fully-qualified Rust type (e.g., `crate::Message`).
- `state_type`: Fully-qualified Rust type for app state (e.g., `crate::AppState`).
- `layout_files`: Optional list or glob of layout files to load.
- `format_output`: Boolean, whether to run rustfmt (default: true).

Parse using the `toml` crate into a `ProjectConfig` struct.

### 2.2 Project State
Create a `Project` struct that holds:
- `config: ProjectConfig`
- `layout: LayoutDocument`
- `node_index: HashMap<ComponentId, Vec<usize>>`
- `selected_id: Option<ComponentId>`
- `history: History` (for undo/redo)

Provide methods:
- `new(path: &Path, template: Option<Template>) -> Result<Project>`: Create new project with default/template layout.
- `open(path: &Path) -> Result<Project>`: Locate config, load layout(s), build node index.
- `save(&self) -> Result<()>`: Write layout back to disk.
- `export(&self) -> Result<()>`: Generate Rust code to `output_file`.
- `find_node(&self, id: ComponentId) -> Option<&LayoutNode>`: O(1) lookup via index.
- `find_node_mut(&mut self, id: ComponentId) -> Option<&mut LayoutNode>`: Mutable lookup.

---

## Phase 3: Undo/Redo (History)

### 3.1 Approach
Use a simple snapshot-based undo stack:
- `History { undo_stack: Vec<LayoutDocument>, redo_stack: Vec<LayoutDocument> }`

### 3.2 Operations
- `push(snapshot: LayoutDocument)`: Called before any mutation; clears redo stack.
- `undo() -> Option<LayoutDocument>`: Pop from undo, push current to redo, return previous.
- `redo() -> Option<LayoutDocument>`: Pop from redo, push current to undo, return next.

### 3.3 Integration
- Wrap every mutating action (add, delete, move, edit property) with a history push.
- Limit stack size (e.g., 50 states) to bound memory usage.

---

## Phase 4: Editor UI – Skeleton

### 4.1 Application State (`App`)
- Fields:
  - `project: Option<Project>`
  - `mode: EditorMode` (Design, Preview)
  - `ui_state`: transient UI state (hovered component, drag state, etc.)

### 4.2 Message Enum
Define a comprehensive `Message` enum covering all user actions:
- `OpenProject`, `SaveProject`, `ExportCode`
- `SelectComponent(ComponentId)`, `DeselectComponent`
- `AddComponent { parent_id: ComponentId, node: LayoutNode }`
- `DeleteComponent(ComponentId)`
- `MoveComponent { id: ComponentId, new_parent: ComponentId, index: usize }`
- `UpdateAttribute { id: ComponentId, attr: AttributeChange }`
- `Undo`, `Redo`
- `SetMode(EditorMode)`
- Internal UI messages: `PaletteHover`, `CanvasDragStart`, `CanvasDrop`, etc.

### 4.3 Top-Level Layout
The `view` function should return an Iced `Row` (or use `iced::widget::pane_grid` for resizable panes):
- Left pane: Widget Palette (fixed or resizable width).
- Center pane: Canvas/Viewport.
- Right pane: Property Inspector (fixed or resizable width).

Initially, implement with simple fixed-width columns; refine with `pane_grid` later.

---

## Phase 5: Widget Palette

### 5.1 Contents
- A vertical list (Column) of labeled buttons or icons, one per widget type.
- Grouped by category:
  - **Containers**: Column, Row, Container, Scrollable.
  - **Widgets**: Text, Button, TextInput, Checkbox, Slider, PickList.

### 5.2 Interaction
- On click: either insert the widget as a child of the currently selected container, or enable a "drag to place" mode.
- For MVP, use click-to-add; implement drag-and-drop in a later phase.

### 5.3 Messages
- `PaletteItemClicked(WidgetKind)` → triggers `AddComponent` if a valid parent is selected.

---

## Phase 6: Canvas / Viewport

### 6.1 Rendering the Layout Tree
- Implement a recursive function: `render_node(node: &LayoutNode, selected_id: Option<ComponentId>) -> Element<Message>`.
- For each node type, produce the corresponding Iced widget using 0.13 syntax:
  - `Column` → `column![...].spacing(n).padding(n)`
  - `Row` → `row![...].spacing(n).padding(n)`
  - `Text` → `text("content").size(n)`
  - `Button` → `button(text("label")).on_press(Message::SelectComponent(id))`
  - `Container` → `container(child).padding(n).width(...).height(...)`
- Wrap every rendered widget in `mouse_area(widget).on_press(Message::SelectComponent(id))` for selection.

### 6.2 Selection Indication
- When a node's ID matches `selected_id`, wrap in a styled container:
  ```rust
  container(widget)
      .style(|_theme| container::Style {
          border: Border {
              color: Color::from_rgb(0.2, 0.5, 1.0),
              width: 2.0,
              radius: 4.0.into(),
          },
          ..Default::default()
      })
  ```
- Consider using `Stack` widget to overlay selection handles without affecting layout.

### 6.3 Disabling Runtime Behavior
- For buttons, do not wire `on_press` to real actions; instead, always emit `SelectComponent`.
- For text inputs, render as read-only or intercept focus to prevent typing.

### 6.4 Scrollable Viewport
- Wrap the entire rendered tree in a `Scrollable` so large layouts can be navigated.

---

## Phase 7: Property Inspector

### 7.1 Dynamic Form Generation
- When `selected_id` is `Some(id)`, look up the node in the layout tree.
- Based on the node's variant, render appropriate input fields:
  - Text content → `TextInput`.
  - Padding/spacing → numeric `TextInput` or `Slider`.
  - Alignment → `PickList` of alignment options.
  - Colors → `PickList` of predefined palette entries (or a simple hex input).

### 7.2 Attribute Changes
- Each input emits `UpdateAttribute { id, attr: AttributeChange::Padding(value) }` (or similar).
- The `update` function locates the node by ID, applies the change, and pushes history.

### 7.3 No Selection State
- If nothing is selected, display a placeholder message: "Select a component to edit its properties."

---

## Phase 8: Tree View

### 8.1 Purpose
- Provide a hierarchical, collapsible view of the layout (like a DOM inspector).
- Easier to select deeply nested or overlapping components.

### 8.2 Rendering
- Recursively render each node as a labeled row with indentation.
- Clicking a row emits `SelectComponent(id)`.
- Highlight the currently selected row.

### 8.3 Drag-and-Drop Reordering (Later)
- Allow dragging rows to reorder children or move nodes between containers.
- This can be deferred until after core editing works.

---

## Phase 9: Code Generation

### 9.1 Generator Function
- `generate_code(layout: &LayoutDocument, config: &ProjectConfig) -> String`
- Recursively walk the tree and emit Rust code strings.

### 9.2 Output Template
Generate a module like:

```rust
// Auto-generated by Iced Builder – do not edit manually.
// Regenerate by opening this project in Iced Builder.

use iced::widget::{column, row, container, scrollable, text, button, text_input, checkbox, slider, pick_list};
use iced::{Element, Length, Alignment, Color};

use crate::Message;  // from config.message_type
use crate::AppState; // from config.state_type

pub fn view(state: &AppState) -> Element<Message> {
    // <generated widget tree>
}
```

### 9.3 Node-to-Code Mapping
For each `LayoutNode` variant, define how it maps to Iced builder syntax:
- `Column` → `column![child1, child2, ...].padding(p).spacing(s).align_x(Alignment::...)`
- `Row` → `row![child1, child2, ...].padding(p).spacing(s).align_y(Alignment::...)`
- `Text` → `text("content").size(n)`
- `Button` → `button(text("label")).on_press(Message::ButtonName)`
- `TextInput` → `text_input("placeholder", &state.field_name).on_input(Message::FieldNameChanged)`
- `Checkbox` → `checkbox("label", state.is_checked).on_toggle(Message::CheckboxToggled)`
- `Slider` → `slider(min..=max, state.value, Message::SliderChanged)`
- `Container` → `container(child).padding(p).width(Length::...).height(Length::...)`

### 9.4 Binding Resolution
- `value_binding: "username"` generates `&state.username`
- `message_stub: "Submit"` generates `Message::Submit`
- For stateful widgets (TextInput, Checkbox, Slider), generate both state reference and message handler

### 9.5 Formatting
- After generating the string, invoke `rustfmt` via `std::process::Command`.
- If `rustfmt` is not available, emit a warning but still write the unformatted code.

### 9.5 Safe Overwrite
- Only write to the configured `output_file`.
- Optionally back up the previous version (e.g., `layout_generated.rs.bak`).

---

## Phase 10: File I/O

### 10.1 Layout Files
- `load_layout(path: &Path) -> Result<LayoutDocument>`: Detect format (RON/JSON) by extension or content, deserialize.
- `save_layout(path: &Path, layout: &LayoutDocument) -> Result<()>`: Serialize and write.
- Create backup (`.bak`) before overwriting existing files.

### 10.2 Project Opening Flow
1. User triggers "Open Project" (file dialog or CLI argument).
2. Use `rfd::FileDialog::new().pick_folder()` for folder selection.
3. Locate `iced_builder.toml` in the selected folder.
4. Parse config.
5. Load layout file(s) specified in config (or default `layout.ron`).
6. Build node index for O(1) lookups.
7. Populate `Project` and display on canvas.

### 10.3 New Project Flow
1. User triggers "New Project".
2. Use `rfd::FileDialog::new().pick_folder()` to select target directory.
3. Optionally select a template (Blank, Form, Dashboard).
4. Generate default `iced_builder.toml` with sensible defaults.
5. Generate initial `layout.ron` (empty or from template).
6. Open the new project.

### 10.4 File Dialogs
- Use `rfd` crate for native file dialogs:
  - `rfd::FileDialog` for synchronous dialogs
  - `rfd::AsyncFileDialog` if using async (requires tokio/async-std feature)
- On Linux: Uses XDG Desktop Portal by default (GTK/KDE native dialogs)
- On Windows/macOS: Uses native OS dialogs
- Add file filters: `.ron`, `.json` for layout files; folders for project open

---

## Phase 11: Keyboard Shortcuts & UX Polish

### 11.1 Shortcuts
- `Ctrl+Z` → `Undo`
- `Ctrl+Y` / `Ctrl+Shift+Z` → `Redo`
- `Delete` / `Backspace` → `DeleteComponent(selected)`
- Arrow keys → navigate selection in tree view
- `Ctrl+S` → `SaveProject`
- `Ctrl+E` → `ExportCode`

### 11.2 Keyboard Handling
- Use Iced's subscription system with `keyboard::on_key_press` to capture global shortcuts:
  ```rust
  fn subscription(&self) -> Subscription<Message> {
      keyboard::on_key_press(|key, modifiers| {
          match (key, modifiers.command()) {
              (keyboard::Key::Character("z"), true) => Some(Message::Undo),
              (keyboard::Key::Character("y"), true) => Some(Message::Redo),
              (keyboard::Key::Named(keyboard::key::Named::Delete), false) => Some(Message::DeleteSelected),
              _ => None,
          }
      })
  }
  ```

### 11.3 Visual Feedback
- Show a status bar or toast for save/export success/failure.
- Highlight invalid nodes in red in the tree view if validation fails.

---

## Phase 12: Editing Modes

### 12.1 Design Mode (Default)
- All interactions are for layout editing.
- Widgets are non-functional (buttons don't "click", inputs don't accept text).

### 12.2 Preview Mode (Stretch)
- Temporarily render the layout with more realistic behavior.
- Useful for checking hover states, scroll behavior, etc.
- Toggle via a toolbar button or keyboard shortcut.

---

## Phase 13: Testing Strategy

### 13.1 Unit Tests
- **Model**: Round-trip serialization, validation logic, history operations.
- **Codegen**: Snapshot tests comparing generated code against expected output for known layouts.

### 13.2 Integration Tests
- Load a sample project, make edits programmatically, export, and verify the output file.

### 13.3 Manual / Visual Testing
- Maintain a `examples/sample_project/` with a variety of layouts to test rendering and editing.

---

## Phase 14: Distribution & CI

### 14.1 GitHub Actions Workflow
- On push to `main` or on tag:
  - Build release binaries for Linux (glibc and musl), Windows, macOS (x86_64 and aarch64).
  - Upload artifacts to the release.

### 14.2 Release Artifacts
- Per-platform `.tar.gz` / `.zip` containing:
  - Binary
  - `README.md`
  - Example `iced_builder.toml`

### 14.3 Versioning
- Use semantic versioning.
- Update `Cargo.toml` version and tag releases.

---

## Implementation Order (Summary)

| Order | Phase | Milestone | Est. Time |
|-------|-------|-----------|----------|
| 1 | 0 | Project setup, dependencies, directory structure | 1 day |
| 2 | 0.4 | Canvas interaction prototype (risk mitigation) | 2-3 days |
| 3 | 1 | Layout AST types, serialization, validation | 2-3 days |
| 4 | 2 | Project config parsing, Project struct | 1-2 days |
| 5 | 3 | Undo/redo history | 1 day |
| 6 | 4 | Iced app skeleton with three-pane layout | 2-3 days |
| 7 | 5 | Widget palette (click-to-add) | 1-2 days |
| 8 | 6 | Canvas rendering of layout tree with selection | 3-4 days |
| 9 | 7 | Property inspector for selected node | 2-3 days |
| 10 | 8 | Tree view sidebar | 2 days |
| 11 | 9 | Code generation and rustfmt integration | 2-3 days |
| 12 | 10 | File I/O (load/save layout, open/new project) | 2 days |
| 13 | 11 | Keyboard shortcuts and UX polish | 2 days |
| 14 | 12 | Preview mode (stretch) | 2-3 days |
| 15 | 13 | Testing (unit, integration, manual) | 3-4 days |
| 16 | 14 | CI/CD and release automation | 2 days |

**Total estimated time: 4-6 weeks** (solo developer, part-time) or **2-3 weeks** (full-time)

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Iced API changes between versions | Pin to `iced = "0.13"` (minor version); update intentionally after testing. |
| Canvas click interception fails with nested widgets | Prototype early (Phase 0.4) with `MouseArea` wrapping; fall back to tree-view-only selection if needed. |
| Complex drag-and-drop implementation | Defer DnD to a later phase; use click-to-add and tree reordering first. |
| Large layouts cause performance issues | Profile early; consider virtualization or `Lazy` widget if needed. |
| `rustfmt` not available on user's system | Gracefully degrade; emit a warning and output unformatted code. |
| File dialogs fail on some Linux distros | `rfd` falls back to Zenity; document XDG Portal requirement. |
| macOS notarization required for distribution | Document signing process; consider ad-hoc signing for initial releases. |
| Parsing arbitrary Rust view code (future) | Limit to a strict, documented subset; fail gracefully with clear errors. |

---

## Next Steps

1. Update `Cargo.toml` with edition `2021` and all dependencies (iced 0.13, rfd, uuid, etc.).
2. Create the directory structure under `src/`.
3. **Build canvas interaction prototype (Phase 0.4)** to validate MouseArea/selection approach.
4. Implement Phase 1 (Layout AST types) and write serialization tests.
5. Proceed through phases in order, committing after each milestone.

This plan should provide a clear path from the current scaffold to a functional Iced Builder MVP.
