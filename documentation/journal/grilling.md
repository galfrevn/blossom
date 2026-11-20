# Grilling

The interview that followed the brainstorm, started on 2026-11-20 with the `grill-with-docs` skill. Each round asks every question that can be answered without guessing at another answer. Decisions are written into the ADRs and the glossary as they land; this file keeps the answers and what is still open.

The interview was held in Spanish. Quotes are translated.

## Round 1

| Question | Answer | Where it went |
| --- | --- | --- |
| Design for the fleet operator, with the small case as the same product? | "Yes, remember it is a visual layer for Vercel Board!" | ADR 0004, accepted |
| Open on Attention, with five lenses and conversation in the background? | Yes | ADR 0005, accepted |
| Keep our own replica and reject third-party sync engines? | "Vercel Board!" | Unclear. ADR 0003 stays proposed until round 2 |
| Dark theme only for the first version? | "Light and dark mode, it should not cost much." | ADR 0006, consequence rewritten |
| Are the improvements wanted in the third concept polish only? | Yes. The list was not given | ADR 0006 |
| Use "Atalaya" as the working name? | "No, horrible. I will think about it properly later." | No name yet. Later named Blossom, ADR 0014 |
| React with TypeScript, state outside React? | "React. I want software that weighs almost nothing and is super fast and light." | ADR 0007, accepted |
| English only for the product? | "English for now, but let's handle i18n if possible, writing only the English dictionary." | ADR 0007, consequence |
| Apache-2.0? | Yes | ADR 0008, accepted |

## Round 2

| Question | Answer | Where it went |
| --- | --- | --- |
| Board as the only source of truth, with a local copy kept only to be instant? | "Whatever you consider best technically. I like the copy." | ADR 0003, accepted |
| Fix "weighs almost nothing" as three budgets to validate in the first spikes? | Yes | ADR 0009 |
| A typed dictionary with only English, resolved at build time, no i18n library at run time? | Yes | ADR 0007, consequence |
| Attention shows only blocked tasks, failed tasks and mentions in the first version? | Yes | ADR 0005, consequence |
| Each person sees only what is theirs by default, with a switch for everyone's? | Yes | ADR 0005, consequence; glossary |
| People can reply, react, unblock, reopen, reassign and cancel, but not claim or complete? | Yes | ADR 0010 |
| Build phase 1 against a local board, connect Vercel's hosted board in phase 2? | Yes | ADR 0002, first release |
| First deliverable is the macOS desktop app? | "Let's start with only the desktop app for macOS." | ADR 0002, first release |

## Round 3

| Question | Answer | Where it went |
| --- | --- | --- |
| Claude takes purely technical decisions in the core and records them as ADRs for review? | Yes | ADR 0011 is the first |
| Fix the speed targets too? | Yes | ADR 0009 |
| Offline: read and search everything, writes disabled rather than queued? | Yes | ADR 0012 |
| One board open at a time, with a switcher? | Yes | ADR 0012 |
| Notify only when something enters Attention, with a Dock badge? | Yes | ADR 0012 |
| First release ships Attention, Flow, Tasks and Fleet; Graph in the second? | Yes | ADR 0012 |
| No telemetry and no connection other than to the board? | Yes | ADR 0013 |
| Signed and notarized distribution through GitHub Releases and Homebrew? | "Yes, but we will see about that later! I do not have and do not want to pay 99 USD for now." | ADR 0012, distribution left unsettled |
| Leave the list of improvements for the next prototype round? | Yes | Next prototype |

## Round 4

| Question | Answer | Where it went |
| --- | --- | --- |
| Can a person create a task and start a thread? Bulk creation stays in the CLI | Yes | ADR 0010 |
| A person always writes as themselves, with no profile switcher? | Yes | ADR 0010 |
| Board administration stays out of the first version? | Yes | ADR 0012 |
| Threads render everything agents post, but replies cannot attach files yet? | Yes | ADR 0012 |
| First run finds a local board by itself, or shows how to start one? | Yes | ADR 0012 |
| The repository stays local or private until the project has a name? | Yes | Roadmap |
| Prototype first, then three spikes, then the app? | Yes | [../plan/roadmap.md](../plan/roadmap.md) |

After this round no question was left that could be asked without guessing.

## Still open

- How the app is distributed on macOS.
- How much history is copied when a board is first connected (technical, waits for the sync spike).
- The improvements wanted in the third concept, to be collected against the fourth.
