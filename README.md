# Iced Builder

Iced Builder is a simple GUI application that allows a developer to visually design and modify the layout of an Iced application's UI and export that layout as Rust/Iced code.

## Features (MVP)
- Visual layout editing using Iced widgets (Columns, Rows, Containers, etc.).
- Widget palette, canvas, and property inspector.
- Undo/redo, tree view of the layout, and basic validation.
- Serialization of layouts to JSON/RON and code generation to a Rust module.

For a detailed architectural overview and roadmap, see `scope.md`.

## Installation

Iced Builder is intended to be distributed as a **standalone portable binary** per platform.

### Linux
1. Download the appropriate archive, for example:
   - `iced-builder-linux-x86_64.tar.gz`
2. Extract it:
   ```bash
   tar -xzf iced-builder-linux-x86_64.tar.gz
   ```
3. Make the binary executable (if needed):
   ```bash
   chmod +x iced-builder
   ```
4. (Optional) Move it somewhere on your `PATH`:
   ```bash
   sudo mv iced-builder /usr/local/bin/
   ```

### Windows
1. Download the `iced-builder-windows-x86_64.zip` archive.
2. Extract it.
3. Run `iced-builder.exe` directly or from a terminal.

### macOS
1. Download the `iced-builder-macos-<arch>.tar.gz` archive.
2. Extract it:
   ```bash
   tar -xzf iced-builder-macos-<arch>.tar.gz
   ```
3. Make the binary executable (if needed) and run it:
   ```bash
   chmod +x iced-builder
   ./iced-builder
   ```

## Runtime Requirements
- No Rust toolchain is required to *run* the application.
- For formatted code generation, `rustfmt` should be installed and available on `PATH`.

## Basic Usage
1. Start `iced-builder`.
2. Use **Open Project** to select a Rust/Iced project folder.
3. If an `iced_builder.toml` and layout files exist, the current layout will be loaded and shown on the canvas.
4. Modify the layout visually, then export to update the configured Rust output file (e.g., `src/ui/layout.rs`).

For configuration details and advanced behavior (layout AST, import/export, future enhancements), refer to `scope.md`.
