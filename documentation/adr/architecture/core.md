---
adr: 0002
status: accepted
date: 2026-11-20
---

# A Rust core with a single web UI

We want an interface that feels instant and that can also open in a browser and be self-hosted. The product is split in two: a core in Rust (protocol client, replica, search index and server) and one web UI that talks to it over IPC on desktop or over WebSocket in a browser. It ships as a single binary, as a desktop app through Tauri, and as a Docker image.

The main reason is that perceived speed comes from reading locally, not from the rendering engine. Also, a board's credentials cannot live in the browser, so a process always has to sit next to the page. We prefer that it be one small, fast process.

## First release

The first release is the desktop app for macOS only. The browser binary and the Docker image come later, on the same core. Development starts against a local board from `vb serve`; Vercel's hosted board is connected next, and its limits shape the design from the first day even though it is not the first to be connected.

## Considered options

- **Pure web** (Next or Vite on the official Node SDK). The shortest path and the easiest to contribute to. Rejected because it does not differ from the community wrappers that appeared on day one.
- **Fully native with GPUI.** The fastest. Rejected because it does not cover the browser or self-hosting well, it narrows contributors to people who know Rust and GPUI, and the governance of GPUI is uncertain.

## Consequences

- There are two languages to maintain. Almost anyone can work on the UI; the core needs Rust.
- Tauri uses the system webview, which behaves differently on each platform (WebKit on macOS and Linux). While the release is macOS only there is one webview to test; all three have to be tested once other platforms are added.
- A terminal UI or a native shell can be added later on the same core without rewriting it.
- The UI reaches the core through one narrow interface, even while the desktop app is the only shell, so that adding the browser later does not mean rewriting the UI.
