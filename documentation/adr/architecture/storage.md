---
adr: 0011
status: accepted
date: 2026-11-20
---

# SQLite for the replica and for search

Purely technical choices in the core were delegated to Claude, to be recorded here for review. This is the first: the replica is stored in SQLite, one file per board, and search uses SQLite's own full-text index rather than a separate search engine.

SQLite is embedded, so it adds no process and little weight, which the budgets in ADR 0009 require. It is also what Board's own local backend uses, so the shape of the data is known to fit. One engine for both storage and search keeps the installer small and gives search and data a single transaction.

## Considered options

- **A dedicated search library** such as Tantivy. Better ranking and faster on very large corpora, at the cost of a second store to keep consistent and more weight in the binary. Worth revisiting only if search quality turns out to be a problem.
- **A key-value store.** Lighter still, but every lens is a query over tasks, agents and messages, and writing those queries by hand is the cost.

## Consequences

- Each lens is a query. Slow lenses are fixed with indexes, not with caches in the UI.
- How much history is copied when a board is first connected is still open. It will be decided with the sync spike, since it depends on how fast Vercel's hosted board can be read one channel at a time.
