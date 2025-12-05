Simple GUI application that allows a developer to visually design and modify the layout of an Iced application's UI and export that layout as Rust/Iced code.

## Tech Stack
- **Rust**: Core language for the application and code generation (edition 2021).
- **Iced 0.13.x**: GUI framework used both by the builder and the generated UIs. Pin to a specific minor version to avoid breaking API changes.
- **serde** (+ `serde_json` or `ron`): Serialization of the layout AST and configuration files.
- **Config format**: `RON`, `JSON`, or `TOML` (for `iced_builder.toml`).
- **rfd 0.15+**: Cross-platform native file dialogs (supports XDG Desktop Portal on Linux, native dialogs on Windows/macOS).
- **uuid**: Unique component ID generation.
- **rustfmt**: Formatting of generated Rust source code (invoked via `std::process::Command`).
- **Error handling crates**: `thiserror` for custom error types, `anyhow` (optional) for prototyping error propagation.

## Target Users & Workflow
- **Users**: Rust/Iced developers who want a faster way to prototype and iterate on UI layouts.
- **Typical Workflow**:
	- Start from a blank layout or from a template (e.g., "Form", "Dashboard").
	- Add and arrange widgets visually in a canvas.
	- Adjust properties (alignment, spacing, style, etc.) using an inspector.
	- Save the layout as a project asset and/or export a Rust module.
	- Switch back to the code editor to wire up business logic and messages.

## Supported Iced Subset (MVP)
- **Layout containers** (from `iced::widget`):
	- `Column`, `Row`, `Container`, `Scrollable`, `Stack` (for overlays).
- **Basic widgets** (from `iced::widget`):
	- `Text`, `Button`, `TextInput`, `Checkbox`, `Slider`, `PickList`, `Space` (for layout spacing).
	- Note: Only a subset may be enabled in the first iteration; prioritize `Text`, `Button`, `Column`, `Row`, `Container`.
- **Interaction wrapper**:
	- `MouseArea` – used internally to intercept clicks for component selection in design mode.
- **Layout properties** (mapped to Iced methods):
	- `padding` → `.padding(...)`, `spacing` → `.spacing(...)`
	- `align_items` → `.align_x(...)` / `.align_y(...)`
	- `width`/`height` → `.width(Length::...)` / `.height(Length::...)`
	- Length values: `Fill`, `Shrink`, `FillPortion(u16)`, `Fixed(f32)`
- **Styling**:
	- Font size (`.size(...)`), limited color palette via `iced::Color`, basic alignment.
	- Use Iced's built-in `Theme` enum (e.g., `Theme::Dark`, `Theme::Light`, `Theme::TokyoNight`).
- **Known limitations (MVP)**:
	- No custom widget types initially.
	- No `Task` (async command) handling or complex state management.
	- Message wiring is stubbed (e.g., `Message::ButtonPressed`, `Message::InputChanged(String)`) and left for the developer to implement.
	- No `Subscription` support for external events.

## 1. Data Model ("Layout AST")
An intermediate data structure will represent the UI tree instead of directly editing Rust source.

- **Component Tree**:
	- A recursive struct/enum describing the layout hierarchy (e.g., `Column`, `Row`, `Button`, `Text`, etc.).
	- Each node has a unique `ComponentId` (UUID-based) for stable identification.
- **Attributes**:
	- Per-component configuration such as padding, spacing, text content, size, style, and alignment.
	- Default values: `padding: 0`, `spacing: 0`, `width: Shrink`, `height: Shrink`, `align_items: Start`.
- **Bindings & Message Stubs**:
	- `value_binding`: A string field name (e.g., `"username"`) that maps to `&state.username` in generated code.
	- `message_stub`: A message variant name (e.g., `"Submit"`) that generates `Message::Submit` or `Message::Submit(value)`.
	- The builder does NOT generate `AppState` or `Message` enums—users define these manually. The generated `view` function references them.
- **Node Index**:
	- Maintain a `HashMap<ComponentId, NodePath>` for O(1) lookup by ID when the tree is deeply nested.
- **Selection State**:
	- Tracks which `ComponentId` is currently selected so the inspector can edit its properties.
- **Serialization**:
	- The tree should be easily serializable (e.g., `serde` + JSON/RON) to store layouts on disk.
- **Validation Constraints**:
	- Leaf nodes (Text, Button, etc.) cannot have children.
	- Bindings and message stubs must be valid Rust identifiers (regex: `^[a-zA-Z_][a-zA-Z0-9_]*$`).
	- Circular references are impossible by tree structure, but moves must be validated to prevent container-into-self.

## 2. Editor UI Layout
The builder itself is an Iced application with an IDE-like layout:

- **Left Sidebar – Widget Palette**:
	- List of available Iced widgets and layout containers the user can add to the scene.
	- Grouped by category: Containers, Widgets.
- **Center – Canvas / Viewport**:
	- Renders the `Component` tree using actual Iced widgets wrapped in `MouseArea` for click interception.
	- Selection is indicated via `Container::style` with a colored border.
	- Buttons emit `SelectComponent(id)` instead of their normal `on_press` action.
	- TextInputs are rendered read-only or with intercepted focus.
- **Right Sidebar – Property Inspector**:
	- Allows editing properties of the selected component:
		- Layout: padding, spacing, alignment, width/height (with `Length` variant picker).
		- Content: text value, labels, placeholder strings, binding names.
		- Style: font size, color choices from a predefined palette.
- **Bottom (optional) – Tree View**:
	- Hierarchical, collapsible view of the component tree (like a DOM inspector).
	- Can be toggled or docked to the left sidebar.

## 3. Core Features
- **Drag and Drop**:
	- Drag widgets from the palette onto the canvas.
	- Reorder children within containers (`Column`/`Row`), possibly via drag handles or up/down controls.
- **Tree View**:
	- Hierarchical representation of the component tree (similar to a DOM inspector) for precise selection.
- **Undo/Redo**:
	- Maintain a stack of layout states for undo/redo of operations (add, remove, move, edit properties).
- **Validation**:
	- Ensure the tree remains valid (e.g., leaf widgets cannot have children).
	- Provide error or warning messages before export if something is not representable as Iced code.

## 4. Persistence & Project Integration
- **Layout Files**:
	- Store layouts as `layout.json` or `layout.ron` in the project, or as multiple files in a `layouts/` directory.
	- Default: single `layout.ron` at project root.
- **Project Config** (e.g., `iced_builder.toml`):
	- `project_root`: path to the target project (optional, defaults to config file location).
	- `output_file`: Rust file path to write generated UI code (e.g., `src/ui/layout_generated.rs`).
	- `message_type`: fully-qualified Rust type name for messages (e.g., `crate::Message`).
	- `state_type`: fully-qualified Rust type name for app state (e.g., `crate::AppState`).
	- `layout_files`: optional list or glob of layout files to load.
- **Safe Code Integration**:
	- Only overwrite generated modules (e.g., `layout_generated.rs`).
	- Encourage a pattern where user-written code wraps or uses generated layout modules, keeping user logic separate.
	- Optionally create a backup (`.bak`) before overwriting.

### 4.1 Project Opening & Layout Import
- **Open Project Folder**:
	- The builder can open a Rust/Iced project folder and locate an `iced_builder.toml` file.
- **Primary Import Path (MVP)**:
	- If layout files (`layout.ron`, `layout.json`, or files in `layouts/`) exist, load them into the internal Layout AST and render them on the canvas.
- **Source-Aware Integration (Future)**:
	- Optionally use Rust parsing (e.g., `syn`) to inspect configured view files and reconstruct a Layout AST from a constrained subset of Iced builder patterns (e.g., `Column::new().push(...)`, `Text::new("...")`).
	- If parsing fails or encounters unsupported patterns, fall back to starting from a blank/new layout or existing layout files.

## 5. Editing Modes & UX
- **Design Mode**:
	- All interactions are for selecting and editing layout; widget actions do not perform application logic.
	- Show selection outlines and possibly simple resize/spacing handles.
- **Preview Mode** (later enhancement):
	- More closely mimics actual widget behavior while still not tied to full app logic.
- **Keyboard Shortcuts**:
	- `Ctrl+Z` / `Ctrl+Y` for undo/redo.
	- Arrow keys to move selection in the tree.
	- `Delete` to remove the selected component.

## 6. Code Generation
- **From Layout AST to Rust**:
	- A generator function walks the layout tree and produces Rust/Iced code.
	- Generates a `view` function: `pub fn view(state: &AppState) -> Element<Message>`.
	- Uses Iced 0.13 syntax: `column![...]`, `row![...]`, `text(...)`, `button(text(...)).on_press(...)`.
- **Generated Imports**:
	```rust
	use iced::widget::{column, row, container, scrollable, text, button, text_input, checkbox, slider, pick_list};
	use iced::{Element, Length, Alignment};
	```
- **Binding Resolution**:
	- `value_binding: "username"` → `&state.username` in generated code.
	- `message_stub: "Submit"` → `Message::Submit` (user must define this variant).
	- `message_stub: "InputChanged"` for TextInput → `Message::InputChanged(String)`.
- **Output Structure**:
	- Single file (e.g., `layout_generated.rs`) consumed by a hand-written wrapper.
	- MVP does NOT generate `AppState` struct or `Message` enum—users define these.
- **Formatting**:
	- Invoke `rustfmt` via `std::process::Command::new("rustfmt").arg("--emit=stdout")`.
	- If `rustfmt` is not available, emit a warning and write unformatted code.

## 7. Future Enhancements (Stretch Goals)
- **Import from Existing Iced Code** (Complexity: Hard):
	- Parse existing Iced Rust code to extract widget structure and import into the Layout AST.
	- Would use `syn` crate for Rust AST parsing, targeting simple `column!/row!` macros and builder patterns.
	- Challenges: Rust's flexible syntax, conditional logic, dynamic state references, and project-specific types.
	- Feasible subset: Static widget trees without conditionals or loops (~60% coverage).
	- Full implementation estimate: 1-3 months depending on coverage goals.
- **Multiple Screens / Routes**:
	- Support multiple root layouts and generate an enum or routing structure for them.
- **Theming Support**:
	- Define themes and style palettes in the builder; export theme configuration alongside layout code.
- **Re-import from Generated Code**:
	- Parse previously generated layouts back into the AST so they can be re-opened and edited.
	- Simpler than general import since generated code follows known patterns.

## 8. Distribution & Releases
- **Standalone Portable Binary**:
	- The application is distributed primarily as a single, standalone binary per supported platform (Linux, Windows, macOS), with no installer required.
- **Per-Platform Artifacts**:
	- Build release binaries via `cargo build --release` for each target.
	- Package in `.tar.gz` (Linux/macOS) or `.zip` (Windows) archives with a minimal README and example `iced_builder.toml`.
- **Platform-Specific Notes**:
	- **Linux**: Uses XDG Desktop Portal for file dialogs (requires portal backend installed). Test on both X11 and Wayland.
	- **Windows**: Add `#![windows_subsystem = "windows"]` to prevent console window. May need application manifest for modern visual styles.
	- **macOS**: May require notarization and entitlements for distribution outside the App Store. Test on both Intel and Apple Silicon.
- **Runtime Requirements**:
	- Rust toolchain is not required to *run* the binary.
	- If code formatting is enabled, `rustfmt` must be installed and available on `PATH` in the user environment.

## 9. New Project Workflow
- **New Project**:
	- User can start a new project from scratch without opening an existing folder.
	- Creates a default `iced_builder.toml` and empty `layout.ron`.
	- Optional: Select from built-in templates ("Blank", "Form", "Dashboard").
- **Templates**:
	- Stored as embedded RON layout files within the binary.
	- Users select a template on project creation; it populates the initial layout.