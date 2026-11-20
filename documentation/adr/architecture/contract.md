---
adr: 0018
status: accepted
date: 2026-11-20
---

# The UI reaches the core through one contract, generated from Rust

ADR 0002 asks for one narrow interface between the UI and the core, even while the desktop app is the only shell. The core exposes two kinds of call and knows nothing about how they travel: requests, which answer once, and subscriptions, which push changes until they are dropped. The desktop shell carries requests as Tauri commands and subscriptions over Tauri's IPC channels. The TypeScript types for both are generated from the Rust types into a single file, so they are never written twice by hand.

## Consequences

- The UI talks to the core through one client module. It has one implementation, over Tauri; the WebSocket one comes with the browser binary.
- The same seam lets the UI run in an ordinary browser against a fake core. That is how it is tested end to end, since Tauri's WebDriver support does not cover macOS.
- Whether the UI holds rows or asks the core for windows of them is left to the list spike: 100,000 messages kept in JavaScript would likely break the memory budget of ADR 0009.
- The generator, specta or ts-rs, is chosen when the contract is built. It has to write a single file and add nothing at run time.
