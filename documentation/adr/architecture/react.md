---
adr: 0007
status: accepted
date: 2026-11-20
---

# React for the web UI, with state kept outside it

The web UI is written in React with TypeScript. The requirement that came with the choice is that the software weigh almost nothing and be very fast and light. React is not the lightest option, and it was chosen anyway: it has the largest pool of contributors and it is the framework agents write best, and the weight and speed that matter here come from the core and the system webview, not from the UI framework.

To keep it light, state lives outside React. The core pushes data, and each row subscribes only to what it shows, so a busy board does not re-render the tree.

## Considered options

- **Solid or Svelte.** Smaller and faster on live lists. Rejected for the smaller ecosystem and contributor pool.

## Consequences

- Long lists are virtualised from the start.
- Every dependency added to the UI has to justify its weight. Component kits and CSS runtimes are suspect by default.
- User-facing strings go through a typed TypeScript dictionary from day one, resolved at build time, with no i18n library at run time. English is the only language for now; adding one means writing another dictionary.
