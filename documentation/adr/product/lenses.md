---
adr: 0005
status: accepted
date: 2026-11-20
---

# One log, several lenses, with Attention first

The official inspector answers "what was said?" with a chat and a task list. With many agents working, the question a person has is different: "what needs me, and is the fleet healthy?". The product presents the same board through several lenses (Attention, Flow, Tasks, Graph and Fleet, with Replay and schema tables later) and opens on Attention, because human attention is the scarce resource. Color is reserved for the state of a task or an agent; the chrome stays neutral.

Conversation is deliberately in the background. An ordinary message in a thread, with no mention and no block, does not appear on the opening screen; it is only visible in Flow.

## Considered options

- **Chat first**, as the inspector and Slack do. Rejected as the default because a blocked task, a failure and a mention become three more lines in a stream that nobody can read at fleet scale.

## Consequences

- A lens changes how a board is presented and never what it contains, so every lens has to be derivable from the same data.
- Attention shows three things in the first version: blocked tasks, failed tasks, and mentions. A task whose lease keeps expiring is not there yet; it is visible in Fleet and Flow, and "stalled" joins Attention once real use shows where the threshold belongs.
- Attention is personal. By default a person sees only what is theirs (tasks they own and mentions of them), with a switch to see everyone's. Showing everyone's by default would turn it back into a feed.
- Every action needs a keyboard shortcut, and everything has to be reachable from the command palette.
