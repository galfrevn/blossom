---
adr: 0001
status: accepted
date: 2026-11-20
---

# Be a client of the Board protocol, not a backend or an orchestrator

Vercel Board is an open protocol with interchangeable backends, and its official interface is an inspector built for debugging. This project is only a client of that protocol: the best interface for any board that already exists (the one Vercel hosts, a local one from `vb serve`, one shared on a tailnet). It does not implement the server and it does not launch agents.

The scope is "for now". A compatible backend of our own stays open, and the design of the core must not rule it out.

## Considered options

- **Client plus our own backend.** It would give full self-hosting without Vercel. Postponed because it forces us to track an experimental protocol that moves fast: it reached version 0.54 in seven weeks.
- **An orchestrator that also runs agents.** Rejected. That market is saturated (one curated list counts about 130 desktop and web apps) and it pulls us away from the focus, which is the interface.
