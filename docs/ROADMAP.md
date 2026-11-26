# CottonMD Roadmap

## Phase 0 — Foundation & Cross-Platform MVP (Completed)
- [x] Modular 8-crate Cargo workspace (`vault`, `markdown`, `graph`, `bases`, `canvas`, `search`, `cli`, `desktop`)
- [x] Pure-Rust `egui` desktop shell with Dark/Light themes, Split Editor/Preview, Quick Switcher, Search, Tags, Inspector
- [x] Interactive force-directed Graph View, filterable Bases Table with CSV export, and JSON Canvas 1.0 viewer
- [x] Native macOS `.app` bundle + Retina Liquid Glass icon (`AppIcon.icns`)
- [x] Windows x64 support (`windows_subsystem`, `%APPDATA%`, `AppIcon.ico`, PowerShell bundler, Inno Setup installer) & GitHub Actions release workflow

## Phase 1 — Realtime Indexing & Full File Formats
- [ ] Live filesystem watcher (`notify`) for incremental re-indexing on external edits
- [ ] Full `.base` file parser/serializer and formula expression engine
- [ ] Editable JSON Canvas nodes/edges with save-back to `.canvas`
- [ ] Upgrade `cottonmd-search` to Tantivy + Nucleo fuzzy scoring
- [ ] Link-aware file rename & move (`cottonmd-cli` + desktop)

## Phase 2 — Rich Editing & Rendering
- [ ] Rope text buffer (`ropey`) + syntax-highlighted Markdown editing + optional Vim keybindings
- [ ] Embedded diagram rendering (Mermaid-compatible SVG)
- [ ] LaTeX math rendering & integrated PDF viewer

## Phase 3 — Extensibility & Ecosystem
- [ ] Capability-sandboxed WASM plugin runtime (`wasmtime`)
- [ ] OS Keychain secret store (`keyring`)
- [ ] Optional end-to-end encrypted sync & macOS QuickLook extension
