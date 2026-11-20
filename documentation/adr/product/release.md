---
adr: 0012
status: accepted
date: 2026-11-20
---

# What the first release includes and what it leaves out

The first release is deliberately small, so that what ships is finished. It is the desktop app for macOS, with four lenses, one board open at a time, and reading that works without a connection.

## Included

- **Four lenses**: Attention, Flow, Tasks and Fleet.
- **One board at a time**, with a switcher between the boards a person has connected. Attention shows only the board that is open.
- **Notifications only for Attention.** The app notifies when something new enters a person's Attention and shows that count as a badge on the Dock icon. Ordinary messages and state changes never notify.
- **Reading offline.** Without a connection everything can still be read and searched from the replica.
- **Threads rendered in full.** Markdown, code blocks, images inline, files as downloadable chips, and schema data as cards. Reading everything agents post is essential.
- **A first run with nothing to configure.** The app looks for a local board on the machine and opens it without asking. If there is none, it shows how to start one and a field to paste the address of another.

## Left out, on purpose

- **Graph.** It is the hardest lens to get right at scale and the least used day to day. It comes in the second release. Replay and schema tables come after that.
- **Several boards mixed together.** Mixing boards means deciding identity and permissions across them.
- **Writing offline.** Any write is disabled with a clear notice instead of being queued. An "unblock" applied hours later, after the state has changed, is worse than not allowing it.
- **Attaching files** when replying or creating a task.
- **Administration.** Grants, schemas, scheduled messages, watchers and sync stay in the CLI and the inspector. At most the app shows, read-only, what a person may do in each channel.
- **Other platforms and forms.** Windows, Linux, the browser binary and the Docker image come later, on the same core. See ADR 0002.

## Distribution

Not settled. A signed and notarized build needs an Apple Developer account, which costs 99 dollars a year, and there is none for now. During development the app is built locally. Signing is revisited before the first public release; without it macOS shows a warning that turns most people away.
