# Roadmap

The order of work agreed on 2026-11-20, and changed the same day when scaffolding moved ahead of the spikes. This is a plan, not a decision record: it is cheap to change, and it should be changed when a step teaches something.

## 1. Scaffolding

The skeleton that the spikes and the app are built on: the workspace, the conventions checked by machine, the desktop shell with the UI inside it, the contract between them, and CI with the first budgets. Its shape is in ADRs 0017 to 0020. It was finished on 2026-11-20: tickets 01 to 05 under `.scratch/scaffold`.

Its first behaviour is the first run of ADR 0012: the app finds a local board and opens it, or shows how to start one. How a local board can be found is in [../research/board.md](../research/board.md).

## 2. Prototype, fourth concept

Close the visual design before the lenses are built on it. The fourth concept is in English, has light and dark themes, has no glass (ADR 0006), carries no wordmark, and shows only the four lenses of the first release. It is there to be marked up: the improvements still wanted in the third concept are collected against it.

It can run alongside the spikes. The scaffold needs only the fonts and the tokens.

Closed on 2026-11-21: the owner approved the fourth concept as it stands (ADR 0006).

## 3. Three technical spikes

Each one answers a question that could change the plan. They run on the scaffold, inside the real desktop shell.

- **Sync.** Keep a replica in step with a local board from `vb serve`. Settles how much history to copy when a board is first connected. Answered on 2026-11-20 for local boards: copy everything, newest threads first; the measurements are in [../research/board.md](../research/board.md) and the decision in ADR 0011. Built by 2026-11-21: the replica copies a local board, follows it live, notices when it dies and resumes after a restart (`.scratch/sync`).
- **List and budgets.** Render a list of 100,000 messages and measure every number in ADR 0009. Settles whether the UI holds rows or asks the core for windows of them (ADR 0018). Answered on 2026-11-21: the UI asks the core for windows, and the measurements are in [../research/list.md](../research/list.md).
- **Token.** Read or renew the development token for Vercel's hosted board from Rust. Answered on 2026-11-20: ADR 0022.

## 4. The app

Each part is a spec with its tickets under `.scratch/`, written on 2026-11-21. In the order they are built:

1. **Reading, against a local board.**
   - `flow`: the concept's frame, and every message in order. It comes first because the other lenses live in its frame.
   - `thread`: a thread read in full in the detail pane.
   - `tasks`: the replica keeps task events and actors, and the Tasks lens.
   - `fleet`: every agent as one cell.
   - `search`: the finder, the commands and the shortcuts.

   Once Flow's first ticket and the tasks spec's first ticket land, `fleet` and `search` can run alongside the others.
2. **Writing and Attention.**
   - `writing`: the person's actions in ADR 0010, always optimistic, disabled offline.
   - `attention`: what waits on the person, where the app opens, with notifications and the Dock's count.
3. **Boards:** the switcher, reading offline, and opening a board by its address (`boards`).
4. **Vercel's hosted board** (`hosted`). It waits for a hosted board to measure against.
5. **Release** (`release`): a command that measures every budget, and a cold start under 300 ms. The page draws at 120 Hz since 2026-11-21 (ADR 0023); signing waits on the owner.

What the first release contains is in ADR 0012.

## Waiting on something else

- **Distribution on macOS** waits for a decision on an Apple Developer account.
- **A hosted board to measure** waits for the owner to have one (`.scratch/hosted`).
- **Graph**, then Replay and schema tables, come after the first release.
- **Glass** comes back, if it does, once the first release works (ADR 0006).
- **Opening on Attention when a board is nearly empty** waits for real use. A person with one agent who mostly talks to it may need Flow first (ADR 0004, [../research/landscape.md](../research/landscape.md)).
