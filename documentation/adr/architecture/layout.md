---
adr: 0017
status: accepted
date: 2026-11-20
---

# One repository in three parts: core, desktop and ui

The repository holds the whole product. A Cargo workspace has two crates: the core in `core/`, a library that knows nothing about Tauri, and the desktop shell in `desktop/`, a thin Tauri binary that carries the core over IPC. The React UI lives in `ui/`. The browser binary of ADR 0002 will be a third crate, `server/`, on the same core, and it is not created until it is built.

## Consequences

- The crate in `core/` is named `blossom`, because Cargo refuses `core`, the name of Rust's built-in crate. Code reads `blossom::replica::merge`.
- `desktop/` replaces Tauri's default `src-tauri`, and its configuration is `Tauri.toml` rather than `tauri.conf.json`, so both follow the naming rule of ADR 0015.
- `ui` is the one abbreviation in a name. It was chosen over `interface` on purpose; do not rename it.
- The core is one crate with modules. It is split only when a second consumer or compile times ask for it.
- The core never depends on Tauri, so its tests and lints run on any machine. Only the desktop build needs macOS.
