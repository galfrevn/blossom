# Stack

What was found about each candidate stack on 2026-11-20. These are findings from third-party sources and project pages. None of it was measured here, and the GPUI situation in particular was not verified in depth.

## Tauri

- Comparisons published in 2026 report installers of 2 to 10 MB against 80 to 200 MB for Electron, and roughly half the idle memory.
- Tauri 2 also targets iOS and Android from the same codebase.
- The cost: it renders through the system webview. Windows uses a Chromium-based WebView2; macOS and Linux use WebKit, which behaves differently. Teams that ship on Tauri test on every target webview.
- Electron's release tooling (signing, notarization, differential updates) is more mature.

## GPUI

- Discord announced it is rewriting its client in Rust with GPUI, replacing Electron. No public release date.
- `longbridge/gpui-kit` has about 21,000 stars, more than 75 components, virtual lists and tables, a code editor, and a WebAssembly target. It is Apache-2.0.
- Against it: there are reports that Zed reduced its investment in GPUI in 2026. `gpui-kit` builds on snapshots of GPUI (`gpui-pre`) and on Longbridge's own fork (GPUI Fast), so the ecosystem is fragmented.
- The WebAssembly target draws to a canvas, which makes it a poor fit for a self-hosted page.

## Sync engines

Zero, TanStack DB, LiveStore and Electric were looked at. They assume the application owns the database and the backend. Here the source of truth is the Board protocol, which already provides a snapshot, a cursor and events that carry full state. See ADR 0003.

## Fonts

- Geist and Geist Mono are on Google Fonts.
- Geist Pixel is not. It ships in the `geist` package on npm (1.9.2 at the time), under the SIL Open Font License, in five variants: Square, Grid, Circle, Triangle and Line. The concept uses Square.

## Sources

- https://github.com/longbridge/gpui-kit
- https://shattered.io/discord-drops-electron-rust-gpui-phoenix-290-mb-ram-2026/
- https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026
- https://kanopylabs.com/blog/tauri-vs-electron-vs-flutter-desktop-apps
- https://johnny.sh/blog/choosing-a-sync-engine-in-2026/
