---
adr: 0020
status: accepted
date: 2026-11-20
---

# The app's identifier is io.github.galfrevn.blossom

macOS keeps an app's data under its bundle identifier, in `~/Library/Application Support/<identifier>`, and that is where the replicas live. Changing the identifier later strands everyone's data in the old folder, so it is fixed before the first build. The project has no domain of its own, so the identifier is its GitHub address reversed.

## Consequences

- A replica can be rebuilt from its board, but unread state and settings cannot. Changing the identifier means writing a migration that moves the old folder, or losing them.
