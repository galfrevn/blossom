# Brainstorm

Session of 2026-11-20. This is the record of what was discussed, written so a fresh person or agent can pick the work up. Decisions live in the ADRs and findings live under `research`; this file points at them instead of repeating them.

The conversation was held in Spanish. Quotes below are translated.

## The request

Vercel released Board for agents and made it headless. The goal is to plan and build the definitive UI for it, as an open source project for the community: incredibly fast and beautiful, with the best UX of any dashboard or board. Whether it should be native, something in Docker, or self-hostable was left open.

## What happened

1. **Research.** Board turned out to be a message board, not a kanban, and an open protocol with interchangeable backends. See `research/board.md`, `research/landscape.md` and `research/stack.md`.
2. **A brief and a first concept** were published as private artifacts (links below).
3. **Four questions were asked.** The answers:
   - Architecture: "Rust core plus web UI." See ADR 0002.
   - Scope: "Client only, FOR NOW." See ADR 0001.
   - Audience: "The same target as Vercel Board. I understand those are large operators. I'm not sure." See ADR 0004, still proposed.
   - Visual direction: "I'll look for references and come back."
4. **A reference arrived**: a screenshot of the T3 Code desktop app, with "I like this interface style: more modern, minimalist but expensive, Apple style."
5. **The concept was redone twice** against that reference. See the iterations below and ADR 0006.
6. **Documentation was requested**, using Matt Pocock's skills, with these rules: everything in English, single-word file names, folders for divisions, no hyphens.

## Concept iterations

| Version | What it was | Verdict |
| --- | --- | --- |
| 1 | A sober monochrome instrument, light and dark, four example agents | None on style; references were promised instead |
| 2 | Apple-like: large system type, wide radii, a lot of air. Scaled to 240 agents | "I don't like it. Look at the image again. Smaller text, more glass, more dev, Cursor style, more dividers, pixel style, dither." Then: "Use Geist and Geist Pixel." |
| 3 | Dark glass, 13px, dividers everywhere, dithered header, Geist Pixel, status bar | "There are things to improve but it is much better." |

All three share the same structure: five lenses (Attention, Flow, Tasks, Graph, Fleet), a detail pane, a command palette, and keyboard shortcuts. From version 2 on, the data is 240 simulated agents and about 870 tasks that move live, and Attention groups tasks that wait for the same answer.

## Decisions

This session produced ADRs 0001 to 0006. Three were accepted during it (0001, 0002, 0006) and three were left proposed. Their current status is in the index in [../README.md](../README.md), and the interview that settled them is in [grilling.md](grilling.md).

Smaller conventions that did not warrant an ADR:

- Documentation lives in `documentation`, not in `docs`.
- Documents are written in English.
- File names are a single word. Folders divide topics. No hyphens.
- The project has no name. "Atalaya" was a working name Claude proposed during this session; it appears in both artifacts and was later rejected.

## Proposals that were shown but never confirmed

This is how things stood at the end of the session. Most of these were settled in the interview that followed: the budgets are in ADR 0009, the scope of the first release in ADR 0012, and the order of work in [../plan/roadmap.md](../plan/roadmap.md).

- **Performance budget**: under 300 ms from cold start to a usable screen, under 50 ms for any interaction, a list of 100,000 messages scrolling at 120 fps, and optimistic writes everywhere. These are design targets, not measurements.
- **Later lenses**: Replay of the event log, and tables generated from registered schemas.
- **Other features**: several boards open at once, native notifications, a terminal UI.
- **Phases**:
  0. Three technical spikes: a token for Vercel's hosted board from Rust, a sync loop against `vb serve`, and a list of 100,000 messages at 120 fps.
  1. Read-only: Flow, thread, Tasks, command palette and search on a local board.
  2. Writing and Attention: reply, react, label and claim, always optimistic.
  3. The lenses nobody else has: Graph, Fleet, Replay and schema tables.
  4. Distribution: desktop app with notifications, several boards, server mode in Docker with login.

## Risks

- **The protocol moves.** Version 0.54 after seven weeks, with a 224 KB changelog. Mitigation: generate types from `openapi.json` and run CI against `vb serve` on every release.
- **Authenticating from Rust.** Lower than first feared for local and tailnet boards. Untested for Vercel's hosted board. See `research/board.md`.
- **Vercel may improve its inspector.** The defence is what an inspector will not have: a local replica, several backends, and polish.
- **Naming.** The project name should not contain "Vercel". "For Vercel Board" is the safe form.
- **System webviews differ.** See ADR 0002.

## Open questions

The questions this session left open were taken into the interview. See [grilling.md](grilling.md) for the answers and for what is still open.

## Artifacts

Both are private to their owner, and both are written in Spanish.

- Interactive concept, version 3: https://claude.ai/artifact/L5otTnFXqHHyYvu59xpJyz
- Brief with research, architectures and phases, version 2: https://claude.ai/artifact/DEQjTDqRYDbZKXqy356GNy

The brief is out of date in one place: it still says the visual direction is pending.

## Suggested skills

Matt Pocock's skills are installed under `.claude/skills`.

- `grill-with-docs`: to close the three proposed ADRs and the open questions above, updating the ADRs and the glossary as answers land.
- `domain-modeling`: whenever a term or a decision changes.
- `prototype`: for the next round on the concept.
- `to-spec`, then `to-tickets`: once the open questions are settled, to turn phase 0 and phase 1 into work.
- `setup-matt-pocock-skills`: before `to-tickets` or `triage`, to choose an issue tracker. Tell it that domain docs live in `documentation`.
