---
adr: 0019
status: accepted
date: 2026-11-20
---

# Tailwind for styles

The UI is styled with Tailwind 4. It compiles to plain CSS at build time and adds no JavaScript at run time, which keeps it clear of the CSS runtimes ADR 0007 distrusts. The design tokens live once, in its theme, as CSS variables for the dark and the light theme. Agents also write it fluently.

## Considered options

- **Hand-written CSS with tokens.** Full control, and no class lists in the markup. Rejected partly over naming: CSS Modules, the usual way to scope it, need files named `x.module.css`, which break the naming rule of ADR 0015, and unscoped global CSS does not hold up as the UI grows.
