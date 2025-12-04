# Iced Builder - Project Status

**Last Updated:** December 4, 2025

## Current Phase: Phase 0.4 - Early Prototype ✅ RUNNING

### Build Status
- ✅ **Compiles successfully** (`cargo build` passes)
- ✅ **Application launches** without panics
- ⚠️ Dead code warnings (expected - scaffolding for future phases)

---

## Application Features (Phase 0.4)

| Feature | Status | Notes |
|---------|--------|-------|
| Three-pane layout | ✅ | Palette \| Canvas \| Inspector |
| Widget palette | ✅ | Container and Widget sections with buttons |
| Empty canvas view | ✅ | Shows placeholder when no project |
| Inspector panel | ✅ | Shows placeholder when nothing selected |
| Tree view | ✅ | Shows hierarchy below canvas |
| Status bar | ✅ | Shows status messages |
| New Project (Ctrl+N) | ✅ | Creates empty Column layout |
| Keyboard shortcuts | ✅ | Ctrl+N, Ctrl+S, Ctrl+Z, Ctrl+Y, Delete, Escape |

---

## Completed Work

### Phase 0 Deliverables

| Component | Status | File(s) |
|-----------|--------|---------|
| Cargo.toml | ✅ | Dependencies: iced 0.13.1, serde, ron, toml, uuid, rfd, thiserror, anyhow, regex |
| Model Layer | ✅ | `src/model/layout.rs`, `project.rs`, `history.rs` |
| UI Components | ✅ | `src/ui/palette.rs`, `canvas.rs`, `inspector.rs`, `tree_view.rs` |
| Code Generator | ✅ | `src/codegen/generator.rs` |
| File I/O | ✅ | `src/io/layout_file.rs`, `config.rs` |
| Utilities | ✅ | `src/util.rs` (rustfmt, identifier validation) |
| App Skeleton | ✅ | `src/app.rs`, `src/main.rs` |
| Sample Project | ✅ | `examples/sample_project/` |

### Architecture Implemented

```
src/
├── main.rs          # Entry point, Iced bootstrap
├── app.rs           # App struct, Message enum, update/view
├── model/
│   ├── layout.rs    # Layout AST (12 widget types)
│   ├── project.rs   # Project config and state
│   └── history.rs   # Undo/redo (50-state limit)
├── ui/
│   ├── palette.rs   # Widget palette sidebar
│   ├── canvas.rs    # Design canvas with selection
│   ├── inspector.rs # Property editor
│   └── tree_view.rs # Hierarchy view
├── codegen/
│   └── generator.rs # AST → Rust code
├── io/
│   ├── layout_file.rs # RON/JSON file handling
│   └── config.rs    # iced_builder.toml
└── util.rs          # Formatting, validation
```

---

## Upcoming Work

### Phase 1: Core Editor UI
- [ ] Functional three-pane layout (palette | canvas | inspector)
- [ ] Basic widget rendering on canvas
- [ ] Click-to-select with visual feedback
- [ ] Property inspector displays selected widget properties

### Phase 2: Widget Manipulation
- [ ] Drag-and-drop from palette to canvas
- [ ] Drag-to-reorder within containers
- [ ] Delete selected component
- [ ] Undo/Redo integration

### Phase 3: Code Generation
- [ ] Real-time code preview panel
- [ ] Export to .rs file via rfd
- [ ] rustfmt integration

### Phase 4: Persistence
- [ ] Save/Load layout files (RON/JSON)
- [ ] Project configuration (iced_builder.toml)
- [ ] Recent files list

### Phase 5: Polish
- [ ] Keyboard shortcuts
- [ ] Error handling improvements
- [ ] Performance optimization

---

## Known Issues

1. **Dead code warnings** - Message variants and functions for future phases exist but are unused
2. **Inspector lifetime complexity** - Required `'static` labels to avoid borrow issues

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| iced | 0.13.1 | GUI framework |
| serde | 1 | Serialization |
| serde_json | 1 | JSON support |
| ron | 0.8 | RON format |
| toml | 0.8 | Config files |
| uuid | 1 | Component IDs |
| rfd | 0.15 | File dialogs |
| thiserror | 2 | Error types |
| anyhow | 1 | Error handling |
| regex | 1 | Identifier validation |
