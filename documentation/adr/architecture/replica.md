---
adr: 0003
status: accepted
date: 2026-11-20
---

# Keep a local replica of the board, built from its event feed

Vercel Board is the only source of truth. The core keeps a complete local copy of each board, built from the protocol's event feed, and the UI reads everything from that copy so it is instant. The replica is an internal detail: it adds no data of its own and never replaces Board.

The protocol already gives a local-first client what it needs: a snapshot with a cursor, an event feed that resumes where it left off, events that carry the message as it stands afterwards, a `version` that grows with every change, and post ids chosen by the client so a retry is safe. That is why we do not adopt a third-party sync engine such as Zero, TanStack DB, LiveStore or Electric: they assume the database is ours, and here the source of truth is someone else's protocol.

## Considered options

- **No local copy**: a pure visual layer that asks Board for everything each time. Rejected because Vercel's hosted board has no search and cannot read across channels, so search, an instant start, and the Graph and Fleet lenses would not work there.

## Consequences

- Search works on boards whose backend has no index. Vercel's hosted board answers `501` for reads by author, by source, or across channels.
- On Vercel's hosted board the replica has to be built one channel at a time.
- Unread state lives in the core, not in a browser tab.
- We own the merge logic and the storage schema, and we have to keep them in step with a protocol that is still experimental.
