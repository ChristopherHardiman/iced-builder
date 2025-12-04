# Iced Builder – Implementation Plan

This document outlines a phased, step-by-step plan for implementing the Iced Builder application as described in `scope.md`. It focuses on architecture decisions, module responsibilities, data flow, and implementation order—without embedding code.

---

## Phase 0: Project Setup & Tooling

### 0.1 Cargo Configuration
- Set the Rust edition to `2021` (the current stable; `2024` is not yet released).
- Add dependencies:
  - `iced` (latest stable, with features for multi-window if needed later).
  - `serde`, `serde_json`, `ron` for serialization.
  - `toml` for parsing `iced_builder.toml`.
  - `uuid` for generating unique component IDs.
  - `thiserror` for custom error types.
  - `anyhow` (optional) for prototyping error propagation.
- Add a `[profile.release]` section with LTO and size optimizations for portable binaries.

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
- Wrap in a newtype for type safety: `struct ComponentId(Uuid)`.

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
  - Bindings and message stubs should be valid Rust identifiers (regex check).
- Return a list of warnings/errors with paths to the offending nodes.

---

## Phase 2: Project & Configuration

### 2.1 Config File (`iced_builder.toml`)
Define the expected structure:
- `project_root`: String path.
- `output_file`: Relative path for generated Rust code.
- `message_type`: Fully-qualified Rust type (e.g., `crate::Message`).
- `layout_files`: Optional list or glob of layout files to load.

Parse using the `toml` crate into a `ProjectConfig` struct.

### 2.2 Project State
Create a `Project` struct that holds:
- `config: ProjectConfig`
- `layout: LayoutDocument`
- `selected_id: Option<ComponentId>`
- `history: History` (for undo/redo)

Provide methods:
- `open(path: &Path) -> Result<Project>`: Locate config, load layout(s).
- `save(&self) -> Result<()>`: Write layout back to disk.
- `export(&self) -> Result<()>`: Generate Rust code to `output_file`.

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
- For each node type, produce the corresponding Iced widget.
- Wrap every rendered widget in a `MouseArea` (or `button` acting as a selection target) that emits `SelectComponent(id)` on click.

### 6.2 Selection Indication
- When a node's ID matches `selected_id`, apply a visual indicator:
  - A colored border (using `Container::style` with a custom style).
  - Or overlay a semi-transparent highlight.

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

```
// Auto-generated by Iced Builder – do not edit manually.

use iced::widget::{column, row, container, scrollable, text, button, text_input, checkbox, slider, pick_list};
use iced::Element;

use <message_type>;

pub fn view(state: &AppState) -> Element<Message> {
    <generated widget tree>
}
```

- Replace `<message_type>` with the configured type.
- Replace `<generated widget tree>` with the recursive output.

### 9.3 Node-to-Code Mapping
For each `LayoutNode` variant, define how it maps to Iced builder syntax:
- `Column` → `column![child1, child2, ...].padding(...).spacing(...)`
- `Text` → `text("content").size(...)`
- `Button` → `button(text("label")).on_press(Message::Stub)`
- And so on.

### 9.4 Formatting
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

### 10.2 Project Opening Flow
1. User triggers "Open Project" (file dialog or CLI argument).
2. Locate `iced_builder.toml` in the selected folder.
3. Parse config.
4. Load layout file(s) specified in config (or default `layout.ron`).
5. Populate `Project` and display on canvas.

### 10.3 File Dialogs
- Use `rfd` crate (or Iced's built-in file dialog if available) to present native open/save dialogs.

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
- Use Iced's subscription system or `keyboard::on_key_press` to capture global shortcuts.

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

| Order | Phase | Milestone |
|-------|-------|-----------|
| 1 | 0 | Project setup, dependencies, directory structure |
| 2 | 1 | Layout AST types, serialization, validation |
| 3 | 2 | Project config parsing, Project struct |
| 4 | 3 | Undo/redo history |
| 5 | 4 | Iced app skeleton with three-pane layout |
| 6 | 5 | Widget palette (click-to-add) |
| 7 | 6 | Canvas rendering of layout tree with selection |
| 8 | 7 | Property inspector for selected node |
| 9 | 8 | Tree view sidebar |
| 10 | 9 | Code generation and rustfmt integration |
| 11 | 10 | File I/O (load/save layout, open project) |
| 12 | 11 | Keyboard shortcuts and UX polish |
| 13 | 12 | Preview mode (stretch) |
| 14 | 13 | Testing (unit, integration, manual) |
| 15 | 14 | CI/CD and release automation |

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Iced API changes between versions | Pin to a specific Iced version; update intentionally. |
| Complex drag-and-drop implementation | Defer DnD to a later phase; use click-to-add and tree reordering first. |
| Large layouts cause performance issues | Profile early; consider virtualization or lazy rendering if needed. |
| `rustfmt` not available on user's system | Gracefully degrade; emit a warning and output unformatted code. |
| Parsing arbitrary Rust view code (future) | Limit to a strict, documented subset; fail gracefully with clear errors. |

---

## Next Steps

1. Fix `Cargo.toml` edition to `2021` and add initial dependencies.
2. Create the directory structure under `src/`.
3. Implement Phase 1 (Layout AST types) and write serialization tests.
4. Proceed through phases in order, committing after each milestone.

This plan should provide a clear path from the current scaffold to a functional Iced Builder MVP.
