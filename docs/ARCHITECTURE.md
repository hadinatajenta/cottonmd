# CottonMD Architecture

CottonMD is organized as a modular Cargo workspace that keeps all domain logic independent of UI toolkits and OS-specific APIs.

## Crate Dependency Graph

```text
           ┌──────────────┐       ┌──────────────────┐
           │ cottonmd-cli │       │ cottonmd-desktop │
           └──────┬───────┘       └────────┬─────────┘
                  │                        │
      ┌───────────┼────────┬───────────────┼──────────┐
      ▼           ▼        ▼               ▼          ▼
┌──────────┐ ┌────────┐ ┌───────┐   ┌──────────┐ ┌────────┐
│  search  │ │ bases  │ │ graph │   │  canvas  │ │        │
└──────────┘ └───┬────┘ └───┬───┘   └──────────┘ │        │
                 │          │                    │        │
                 ▼          ▼                    │        │
           ┌───────────────────┐                 │        │
           │     markdown      │                 │        │
           └─────────┬─────────┘                 │        │
                     │                           │        │
                     ▼                           ▼        ▼
           ┌──────────────────────────────────────────────┐
           │                    vault                     │
           └──────────────────────────────────────────────┘
```

## Core Design Principles

1. **Zero WebView / Electron Overhead:** The desktop app renders directly through `egui` + `eframe` (`glow` OpenGL backend), keeping binaries small (~5–7 MB) and idle CPU usage near zero.
2. **UI-Agnostic Core Crates:** `cottonmd-vault`, `cottonmd-markdown`, `cottonmd-graph`, `cottonmd-bases`, `cottonmd-canvas`, and `cottonmd-search` have minimal dependencies so they can be shared seamlessly by desktop, CLI, and future mobile/QuickLook FFI bindings.
3. **Plain Files First:** Notes live as standard `.md`, `.canvas` (JSON Canvas 1.0), and `.base` files inside ordinary user folders with path-traversal protection enforced inside `cottonmd-vault`.
4. **Cross-Platform Native Conventions:**
   - Config persistence: `~/Library/Application Support/CottonMD` (macOS), `%APPDATA%\CottonMD` (Windows), `$XDG_CONFIG_HOME/cottonmd` (Linux).
   - Native file pickers via `rfd` (Cocoa `NSOpenPanel`, Win32 COM `IFileDialog`, Linux `xdg-desktop-portal`).
   - Adaptive shortcuts (`Cmd` on macOS, `Ctrl` on Windows/Linux).
