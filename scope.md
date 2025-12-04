Simple GUI application that allows a developer to visually design and modify the layout of an Iced application's UI and export that layout as Rust/Iced code.

## Tech Stack
- **Rust**: Core language for the application and code generation.
- **Iced**: GUI framework used both by the builder and the generated UIs.
- **serde** (+ `serde_json` or `ron`): Serialization of the layout AST and configuration files.
- **Config format**: `RON`, `JSON`, or `TOML` (for `iced_builder.toml`).
- **rustfmt**: Formatting of generated Rust source code.
- **Error handling crates** (optional): `thiserror` / `anyhow` for structured and ergonomic error reporting.

## Target Users & Workflow
- **Users**: Rust/Iced developers who want a faster way to prototype and iterate on UI layouts.
- **Typical Workflow**:
	- Start from a blank layout or from a template (e.g., "Form", "Dashboard").
	- Add and arrange widgets visually in a canvas.
	- Adjust properties (alignment, spacing, style, etc.) using an inspector.
	- Save the layout as a project asset and/or export a Rust module.
	- Switch back to the code editor to wire up business logic and messages.

## Supported Iced Subset (MVP)
- **Layout containers**:
	- `Column`, `Row`, `Container`, `Scrollable`.
- **Basic widgets**:
	- `Text`, `Button`, `TextInput`, `Checkbox`, `Slider`, `PickList` (only some may be enabled in the first iteration).
- **Layout properties**:
	- `padding`, `spacing`, `align_items`, `width`/`height`, `horizontal_alignment`, `vertical_alignment`.
- **Styling**:
	- Font size, a limited color palette for text/background, basic alignment.
- **Known limitations (MVP)**:
	- No custom widget types initially.
	- No complex `Command` handling or async wiring.
	- Message wiring is stubbed (e.g., placeholder enum values) and left for the developer to implement.

## 1. Data Model ("Layout AST")
An intermediate data structure will represent the UI tree instead of directly editing Rust source.

- **Component Tree**:
	- A recursive struct/enum describing the layout hierarchy (e.g., `Column`, `Row`, `Button`, `Text`, etc.).
- **Attributes**:
	- Per-component configuration such as padding, spacing, text content, size, style, and alignment.
- **Selection State**:
	- Tracks which component ID is currently selected so the inspector can edit its properties.
- **Serialization**:
	- The tree should be easily serializable (e.g., `serde` + JSON/RON) to store layouts on disk.

## 2. Editor UI Layout
The builder itself is an Iced application with an IDE-like layout:

- **Left Sidebar – Widget Palette**:
	- List of available Iced widgets and layout containers the user can add to the scene.
- **Center – Canvas / Viewport**:
	- Renders the `Component` tree using Iced widgets.
	- Intercepts mouse events so clicks select components instead of triggering their runtime behavior.
- **Right Sidebar – Property Inspector**:
	- Allows editing properties of the selected component:
		- Layout: padding, spacing, alignment, width/height.
		- Content: text value, labels, placeholder strings.
		- Style: font size, color choices, basic theming hooks.

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
- **Project Config** (e.g., `iced_builder.toml`):
	- `project_root`: path to the target project.
	- `output_file`: Rust file path to write generated UI code (e.g., `src/ui/layout.rs`).
	- `message_type`: fully-qualified Rust type name for messages (e.g., `crate::Message`).
- **Safe Code Integration**:
	- Only overwrite generated modules (e.g., `layout_generated.rs`).
	- Encourage a pattern where user-written code wraps or uses generated layout modules, keeping user logic separate.

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
	- A generator function walks the layout tree and produces Rust/Iced code, e.g. a `view` function returning `Element<Message>`.
- **Output Structure**:
	- Optionally split generated code into a dedicated module (e.g., `layout_generated.rs`) consumed by a small, hand-written wrapper.
- **Formatting**:
	- Pipe generated code through `rustfmt` (via command-line or library) to ensure readable, idiomatic source.

## 7. Future Enhancements (Stretch Goals)
- **Multiple Screens / Routes**:
	- Support multiple root layouts and generate an enum or routing structure for them.
- **Theming Support**:
	- Define themes and style palettes in the builder; export theme configuration alongside layout code.
- **Import from Generated Code**:
	- Parse previously generated layouts back into the AST so they can be re-opened and edited.

## 8. Distribution & Releases
- **Standalone Portable Binary**:
	- The application is distributed primarily as a single, standalone binary per supported platform (Linux, Windows, macOS), with no installer required.
- **Per-Platform Artifacts**:
	- Build release binaries via `cargo build --release` for each target and optionally package them in `.tar.gz` / `.zip` archives with a minimal README and example `iced_builder.toml`.
- **Runtime Requirements**:
	- Rust toolchain is not required to *run* the binary.
	- If code formatting is enabled, `rustfmt` must be installed and available on `PATH` in the user environment.