# CottonMD

Lightweight, local-first **Markdown Knowledge Workspace** built in pure **Rust** with **`egui` (`eframe` + `glow`)** — no Electron, no WebView, small native binaries (~6.6 MB `.app` on macOS), and cross-platform support for **macOS, Windows, and Linux**.

## Features

- **Local-First Vault & File Explorer:** Hierarchical folder tree, instant file filter, native OS folder picker (`Open Vault`, `Cmd/Ctrl+O`), `New Note` (`Cmd/Ctrl+N`), and `Quick Switcher` (`Cmd/Ctrl+P` / `Cmd/Ctrl+K`).
- **Markdown Editor, Split & Preview (`Cmd/Ctrl+E`):** Clean writing surface with unsaved indicator, `Save` (`Cmd/Ctrl+S`), and a native preview rendering headings (`H1–H6`), YAML frontmatter properties, callouts (`> [!note]`), fenced code blocks, clickable `[[wikilinks]]` (navigate or create notes) and standard `[text](relative/path.md)` links (navigate notes, open URLs in the browser), `#tags`, and interactive task checkboxes.
- **Interactive Graph View (`Cmd/Ctrl+G`):** Force-directed graph physics rendered directly with `egui::Painter` (pan, zoom, drag nodes, filter, hover highlights, click to open note, zero idle CPU when settled).
- **Bases Table (`Cmd/Ctrl+B`):** Filterable, sortable tabular view over vault notes (folder, tag, property filters) with native CSV export.
- **JSON Canvas Viewer (`.canvas`):** Pan/zoom 2D renderer for JSON Canvas 1.0 documents (Text, File, Link, Group nodes and edges).
- **Inspector Sidebar:** Backlinks, Outgoing Links, Unresolved Links (click to create), Heading Outline, YAML Properties, and interactive Tasks.
- **Dark & Light Themes:** Built-in restrained dark charcoal and warm paper light themes.
- **Link Graph from Wikilinks and Markdown Links:** Backlinks, graph, orphans and unresolved links understand both `[[Note]]` and relative `[text](../folder/note.md)` links, so plain GitHub-style Markdown folders work as a knowledge graph.
- **Vault Scope (`.cottonmd/vault.json`):** Index only chosen folders of a large workspace; `node_modules`, `vendor`, and `target` are always skipped.
- **Companion CLI (`cottonmd-cli`):** Fast headless CLI for scripting vault queries and AI agents (`stats`, `files`, `read`, `search`, `links`, `backlinks`, `related`, `orphans`, `deadends`, `unresolved`, `tags`, `tasks`).

## Workspace Layout

```text
cottonmd/
├── Cargo.toml
├── assets/
│   ├── AppIcon-1024.png   # Liquid Glass macOS squircle master icon
│   ├── AppIcon.icns       # Multi-resolution macOS Retina icon
│   ├── AppIcon.ico        # Multi-resolution Windows icon (16–256px)
│   └── AppIcon-256.png    # Embedded HD runtime window icon (Windows/Linux/dev)
├── docs/
│   ├── ARCHITECTURE.md
│   └── ROADMAP.md
├── scripts/
│   ├── bundle-macos.sh
│   ├── bundle-windows.ps1
│   ├── installer-windows.iss
│   ├── render-liquid-glass-icon.swift
│   └── export-windows-icons.swift
└── crates/
    ├── vault/             # cottonmd-vault
    ├── markdown/          # cottonmd-markdown
    ├── graph/             # cottonmd-graph
    ├── bases/             # cottonmd-bases
    ├── canvas/            # cottonmd-canvas
    ├── search/            # cottonmd-search
    ├── cli/               # cottonmd-cli
    └── desktop/           # cottonmd-desktop (CottonMD)
```

## Build & Install

### macOS
```bash
cargo test --workspace
bash scripts/bundle-macos.sh
open /Applications/CottonMD.app
```

### Windows (PowerShell on Windows 10/11 + Rust MSVC)
```powershell
cargo test --workspace
.\scripts\bundle-windows.ps1
```
Produces `dist\CottonMD-windows-x64.zip` containing `CottonMD.exe` (GUI without console window, embedded `AppIcon.ico`) and `cottonmd-cli.exe`.

Optional Inno Setup 6 Windows installer:
```powershell
iscc .\scripts\installer-windows.iss
```

### Linux
```bash
cargo build --release -p cottonmd-desktop -p cottonmd-cli
```

### CLI Usage
```bash
cargo run --release -p cottonmd-cli -- --vault /path/to/vault stats
cargo run --release -p cottonmd-cli -- --vault /path/to/vault search "architecture"
cargo run --release -p cottonmd-cli -- --vault /path/to/vault related notes/service.md
```

`related` prints one note per line as `in|out<TAB>path<TAB>title`, incoming links first, which makes it a
cheap way for a coding agent to pull every note connected to a page. Without `--vault`, the current
directory is the vault.

### Vault Scope

To use a large workspace (for example a folder of many code repositories) as a vault, add
`.cottonmd/vault.json` at the vault root:

```json
{
  "include": ["_library", "docs"],
  "exclude": ["archive"]
}
```

- `include`: vault-relative folders to index; when empty or missing, everything is indexed.
- `exclude`: directory names skipped at any depth, in addition to `node_modules`, `vendor`, and `target`.
- Unknown keys or invalid JSON are reported as an error instead of being ignored.

## License

Dual-licensed under MIT OR Apache-2.0.
