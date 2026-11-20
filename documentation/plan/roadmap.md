# Roadmap

The order of work agreed on 2026-11-20. This is a plan, not a decision record: it is cheap to change, and it should be changed when a step teaches something.

## 1. Prototype, fourth concept

Close the visual design before building on it. The fourth concept is in English, has light and dark themes, carries no wordmark, and shows only the four lenses of the first release. It is there to be marked up: the improvements still wanted in the third concept are collected against it.

## 2. Three technical spikes

Each one answers a question that could change the plan.

- **Sync.** Keep a replica in step with a local board from `vb serve`. Settles how much history to copy when a board is first connected.
- **List and budgets.** Render a list of 100,000 messages and measure every number in ADR 0009.
- **Token.** Read or renew the development token for Vercel's hosted board from Rust.

## 3. The app

1. Reading, against a local board: Flow, threads, Tasks, Fleet, the command palette and search.
2. Writing and Attention: the actions in ADR 0010, always optimistic, and the Attention lens.
3. Vercel's hosted board.

What the first release contains is in ADR 0012.

## Waiting on something else

- **Distribution on macOS** waits for a decision on an Apple Developer account.
- **Graph**, then Replay and schema tables, come after the first release.
