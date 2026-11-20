---
adr: 0015
status: accepted
date: 2026-11-20
---

# Code explains itself: no comments, one-word files, full-word names

Quality and scale come before speed of delivery in this project, and the code is expected to read on its own. Four rules follow, and they are stricter than what most codebases do: everything is in English; the source has zero comments; every file and folder name is a single lowercase word, with folders doing the dividing; and identifiers are long and descriptive rather than short or abbreviated. The working rules live in `CLAUDE.md`. This record exists so nobody "fixes" them back to the usual conventions.

## Considered options

- **The usual conventions**: hyphenated or camel-cased file names, doc comments on public items, short local names. Rejected in favour of code that has to explain itself through names and structure, because a comment can drift from the code it describes and a name cannot.

## Consequences

- API documentation cannot be generated from doc comments. Anything that needs explaining beyond the code lives in `documentation`.
- A reason that has to outlive the code goes in an ADR, not next to the line.
- Source files carry no license header. The license is stated once, at the root.
- Names a tool dictates (`Cargo.toml`, `package.json`, `README.md`) are the only exceptions to the naming rule. Where a tool lets a name be configured, it is configured: Tauri's default `src-tauri` folder, for example, gets a one-word name.
- Nothing enforces these rules yet. Once there is a toolchain they should be checked by lint and in CI, so they do not depend on anyone remembering them.
