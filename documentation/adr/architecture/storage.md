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
- **A local board is copied whole when it is first connected,** because ADR 0012 promises that everything can be read and searched offline. The sync spike measured it with 100,000 messages: replaying the feed took about 7 seconds, and reading thread by thread about 38. The newest threads are read first, so the app has something to show within a second while the rest arrives. Vercel's hosted board keeps 30 days of feed and lists threads one channel at a time, so its first copy is decided with the token spike. The numbers are in `documentation/research/board.md`.
- **The driver is `rusqlite` with SQLite bundled.** It is synchronous, so the core runs it on its own thread and needs no async pool. It compiles a known SQLite with FTS5 into the binary instead of depending on the system's. The spike stored 100,000 views in 0.9 seconds and read a channel's latest 50 in 0.1 ms. `sqlx` was the alternative, rejected for its async pool and for checking queries against a database at build time, which this project does not need.
