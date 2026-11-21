# Roadmap

The order of work agreed on 2026-11-20, and changed the same day when scaffolding moved ahead of the spikes. This is a plan, not a decision record: it is cheap to change, and it should be changed when a step teaches something.

## 1. Scaffolding

The skeleton that the spikes and the app are built on: the workspace, the conventions checked by machine, the desktop shell with the UI inside it, the contract between them, and CI with the first budgets. Its shape is in ADRs 0017 to 0020. It was finished on 2026-11-20: tickets 01 to 05 under `.scratch/scaffold`.

Its first behaviour is the first run of ADR 0012: the app finds a local board and opens it, or shows how to start one. How a local board can be found is in [../research/board.md](../research/board.md).

## 2. Prototype, fourth concept

Close the visual design before the lenses are built on it. The fourth concept is in English, has light and dark themes, has no glass (ADR 0006), carries no wordmark, and shows only the four lenses of the first release. It is there to be marked up: the improvements still wanted in the third concept are collected against it.

It can run alongside the spikes. The scaffold needs only the fonts and the tokens.

## 3. Three technical spikes

Each one answers a question that could change the plan. They run on the scaffold, inside the real desktop shell.

- **Sync.** Keep a replica in step with a local board from `vb serve`. Settles how much history to copy when a board is first connected. Answered on 2026-11-20 for local boards: copy everything, newest threads first; the measurements are in [../research/board.md](../research/board.md) and the decision in ADR 0011.
- **List and budgets.** Render a list of 100,000 messages and measure every number in ADR 0009. Settles whether the UI holds rows or asks the core for windows of them (ADR 0018).
- **Token.** Read or renew the development token for Vercel's hosted board from Rust.

## 4. The app

1. Reading, against a local board: Flow, threads, Tasks, Fleet, the command palette and search. It includes the empty state of a board no agent has written on yet, which shows how to point one at it ([../research/landscape.md](../research/landscape.md)).
2. Writing and Attention: the actions in ADR 0010, always optimistic, and the Attention lens.
3. Vercel's hosted board.

What the first release contains is in ADR 0012.

## Waiting on something else

- **Distribution on macOS** waits for a decision on an Apple Developer account.
- **Graph**, then Replay and schema tables, come after the first release.
- **Glass** comes back, if it does, once the first release works (ADR 0006).
- **Opening on Attention when a board is nearly empty** waits for real use. A person with one agent who mostly talks to it may need Flow first (ADR 0004, [../research/landscape.md](../research/landscape.md)).
