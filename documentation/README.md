# Documentation

Everything written down about this project so far. The project is called Blossom: an open source visual layer for Vercel Board.

## Layout

```
CLAUDE.md                   how code is written here (at the repository root)
GLOSSARY.md                 the project's language (at the repository root)
documentation/
├── adr/                    decisions, one per file
│   ├── product/
│   ├── architecture/
│   └── design/
├── agents/                 what the skills need to know about this repository
├── plan/                   the order of work
├── research/               findings, with sources
└── journal/                what was discussed, session by session
```

## Conventions

- Documents are written in English.
- File names are a single word. Folders divide topics. No hyphens.
- An ADR carries its number, status and date in its frontmatter, because the file name does not.

## Decisions

| ADR | Decision | Status | File |
| --- | --- | --- | --- |
| 0001 | Be a client of the Board protocol, not a backend or an orchestrator | accepted | [adr/product/scope.md](adr/product/scope.md) |
| 0002 | A Rust core with a single web UI | accepted | [adr/architecture/core.md](adr/architecture/core.md) |
| 0003 | Keep a local replica of the board, built from its event feed | accepted | [adr/architecture/replica.md](adr/architecture/replica.md) |
| 0004 | Design for the fleet operator; the small case is the same product | accepted | [adr/product/audience.md](adr/product/audience.md) |
| 0005 | One log, several lenses, with Attention first | accepted | [adr/product/lenses.md](adr/product/lenses.md) |
| 0006 | Dark, dense, with pixel and dither accents; glass postponed | accepted | [adr/design/visual.md](adr/design/visual.md) |
| 0007 | React for the web UI, with state kept outside it | accepted | [adr/architecture/react.md](adr/architecture/react.md) |
| 0008 | Apache-2.0 for the whole project | accepted | [adr/product/license.md](adr/product/license.md) |
| 0009 | Weight and speed budgets that every change has to respect | accepted | [adr/architecture/budgets.md](adr/architecture/budgets.md) |
| 0010 | People supervise tasks; they do not work them | accepted | [adr/product/supervision.md](adr/product/supervision.md) |
| 0011 | SQLite for the replica and for search | accepted | [adr/architecture/storage.md](adr/architecture/storage.md) |
| 0012 | What the first release includes and what it leaves out | accepted | [adr/product/release.md](adr/product/release.md) |
| 0013 | The app talks only to the board | accepted | [adr/product/privacy.md](adr/product/privacy.md) |
| 0014 | The project is called Blossom | accepted | [adr/product/name.md](adr/product/name.md) |
| 0015 | Code explains itself: no comments, one-word files, full-word names | accepted | [adr/architecture/conventions.md](adr/architecture/conventions.md) |
| 0016 | Soft paper, warm ink, and a cherry blossom accent | accepted | [adr/design/identity.md](adr/design/identity.md) |
| 0017 | One repository in three parts: core, desktop and ui | accepted | [adr/architecture/layout.md](adr/architecture/layout.md) |
| 0018 | The UI reaches the core through one contract, generated from Rust | accepted | [adr/architecture/contract.md](adr/architecture/contract.md) |
| 0019 | Tailwind for styles | accepted | [adr/architecture/styles.md](adr/architecture/styles.md) |
| 0020 | The app's identifier is io.github.galfrevn.blossom | accepted | [adr/architecture/identifier.md](adr/architecture/identifier.md) |
| 0021 | CI checks everything on every push, on free runners, in under two minutes | accepted | [adr/architecture/pipeline.md](adr/architecture/pipeline.md) |
| 0022 | Blossom gets a hosted board's token through the Vercel CLI | accepted | [adr/architecture/token.md](adr/architecture/token.md) |
| 0023 | The page draws at the display's rate, through WebKit's own switch | accepted | [adr/architecture/rate.md](adr/architecture/rate.md) |

"Proposed" means the decision was recommended and shown but not confirmed explicitly. Purely technical decisions in the core were delegated to Claude and are recorded as accepted for review; they say so in their first line.

To add a decision: take the next number, pick the folder it belongs to, name the file with one word, and add a row here.

## Plan

- [plan/roadmap.md](plan/roadmap.md): the order of work, and what is waiting on something else.

## Agents

The rules for writing code are in `CLAUDE.md` at the repository root.

- [agents/skills.md](agents/skills.md): which skill to reach for, and when.
- [agents/domain.md](agents/domain.md): how skills read the glossary and record ADRs.
- [agents/tracker.md](agents/tracker.md): where specs and tickets live.
- [agents/labels.md](agents/labels.md): the triage roles.
- [agents/parallel.md](agents/parallel.md): how several sessions work at once, each in its own folder, with one integrator for every commit.

## Research

- [research/board.md](research/board.md): what Vercel shipped, the protocol, its limits, how a client authenticates and finds a local board, and what copying and following a whole board costs.
- [research/landscape.md](research/landscape.md): community UIs for Board and the neighbouring market.
- [research/list.md](research/list.md): the list spike: windows against the whole list, frames, cold start and memory with 100,000 messages.
- [research/stack.md](research/stack.md): Tauri, GPUI, sync engines and fonts.

## Journal

- [journal/brainstorm.md](journal/brainstorm.md): the session of 2026-11-20, from the request to the third concept.
- [journal/grilling.md](journal/grilling.md): the interview that followed, round by round, with what is still open.

## How this differs from the skills' defaults

The ADR and glossary formats come from the `domain-modeling` skill in Matt Pocock's skills, installed under `.claude/skills`. That skill expects ADRs at `docs/adr/0001-slug.md`. This project keeps them at `documentation/adr/<area>/<word>.md` and puts the number in the frontmatter. `GLOSSARY.md` stays at the repository root, where the skills look for it.
