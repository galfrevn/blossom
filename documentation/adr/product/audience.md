---
adr: 0004
status: accepted
date: 2026-11-20
---

# Design for the fleet operator; the small case is the same product

The project is a visual layer for Vercel Board, so its audience is Board's audience. Vercel's own two cases are at scale (a swarm of security agents, and the v0 factory that merges more than half of its pull requests), while the first contact will be someone trying a local board on their own machine. We design one experience for a person who operates a fleet of hundreds of agents, and the small case inherits it with less data. A fleet has no minimum size: three agents are a fleet.

We accept that someone with three agents sees a sober first screen (no decisions waiting, three cells in Fleet) instead of designing a second experience for them. Two products in one is what makes the design expensive.

## Consequences

- Attention groups tasks that wait for the same answer, so one reply resolves many.
- Fleet is a dense grid where each cell is an agent, not a table of rows.
- Flow summarises repeated progress messages into one line per channel.
- Graph shows only the critical path and folds everything that blocks nobody.
- Every list has to stay fast with thousands of tasks, which is a requirement on the core as much as on the UI.
- Empty and near-empty states need care, because they are what a newcomer sees first.
