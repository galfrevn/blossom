# Attention: what waits on the person looking

**Status:** ready-for-agent

## Problem Statement

With many agents working, the question a person has is "what needs me?", and every other lens answers something else (ADR 0005). Attention is the lens the app opens on. It is personal: by default it shows only what is the person's own. It groups tasks that wait for the same answer, so one reply resolves many (ADR 0004). It is also the only thing that notifies, with a count on the Dock (ADR 0012). It needs the task index (tasks spec) and the person's writes (writing spec).

## Solution

Attention gathers three things: decisions, which are the person's blocked tasks grouped by the answer they need; the person's failed tasks; and mentions of the person. Each is drawn as in the concept, with its actions in reach. A switch shows everyone's instead of the person's. Answering a decision posts the answer in every task's thread and unblocks them all. When nothing waits, it says so and offers Flow with what is new there. The app opens on Attention, notifies when something new enters it, and shows its count on the Dock.

## User Stories

1. As an operator, I want the app to open on what waits on me, so that I start with what only I can do (ADR 0005).
2. As an operator, I want my blocked tasks grouped by the answer they need, so that one reply unblocks them all (ADR 0004).
3. As an operator, I want to answer a decision once and have the answer posted in every task's thread and the tasks unblocked, so that I don't repeat myself across twenty threads.
4. As an operator, I want my failed tasks listed with why they failed, with Reopen and Cancel at hand, so that I can retry or call off each one.
5. As an operator, I want mentions of me listed until I reply or mark them done, so that no question addressed to me gets lost.
6. As an operator, I want a switch to see what waits on anyone on the board, so that I can help others or supervise a team.
7. As an operator, I want an empty Attention to tell me nothing waits and offer Flow with how much is new there, so that an empty screen is good news, not a dead end.
8. As an operator, I want a notification when something new waits on me, and only then, so that the app can sit in the background without nagging (ADR 0012).
9. As an operator, I want the count of what waits on me on the Dock icon, so that I see it without opening the app.
10. As an operator, I want what I have dealt with to leave Attention at once, so that the list is always what is left.
11. As a contributor, I want the gathering and the grouping tested in the core, so that what reaches the person is right.

## Implementation Decisions

- **The person** is the open board's person, from `GET /me`: their principal and their actor ID.
- **Mine:** blocked and failed tasks whose `owner` is the person's principal, and messages whose text mentions the person's actor (`@[actor-id]`). Mentions of the person's own agents are their agents' business, not theirs.
- **A decision** groups the person's blocked tasks with the same `blocked.kind` and the same `reason`, compared after trimming and collapsing whitespace. A task whose reason is its own is a decision of one. Board says what blocked a task but not which answer it waits for, so this rule is a proposal that real use can change.
- **A mention leaves Attention** when the person replies in its thread after it, or marks it done (D). Done is kept in the replica, so it holds across restarts and stays on this Mac (ADR 0003: unread state lives in the core).
- **Answering a decision** posts the answer as a reply in each task's thread and then unblocks each, through the writing spec's writes, optimistic, with each task's failure shown in place.
- **The app opens on Attention.** Opening on Flow when the board is nearly empty waits for real use (roadmap).
- **Notifications** come from the shell when an item enters Mine, after the first copy is done, and never for what was there when the board opened. macOS asks for permission on the first one. The Dock's badge is the count of Mine, cleared at zero. The notification plugin is weighed against ADR 0009's installer budget in its ticket.
- **Stalled agents** are not in Attention yet; they are in Fleet and Flow (ADR 0005).

## Testing Decisions

- The gathering is tested in the core against the fake board's history: blocked and failed tasks of the person and of others, mentions of the person and of their agents, a reply that clears a mention, and a mention marked done.
- The grouping is tested with reasons that match, that differ only in whitespace, and that differ.
- Answering a decision is tested against the fake board: every reply posted and every task unblocked, and one that fails.
- The lens, the switch and the empty state are tested against the fake core.
- Notifications and the badge are verified by hand, since they are the system's.

## Out of Scope

- Stalled agents in Attention (ADR 0005).
- Notifications for anything outside Attention (ADR 0012).
- Several boards in one Attention (ADR 0012).
