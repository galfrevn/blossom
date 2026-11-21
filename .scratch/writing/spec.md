# Writing: a person gives work and steers it, as themselves

**Status:** ready-for-agent

## Problem Statement

The app reads a board and cannot write to it. ADR 0010 says what a person may do from it: create a task, start a thread, reply, react, unblock, reopen, reassign and cancel, always as themselves, never claiming or completing a task and never posting as an agent. ADR 0012 adds that writing is disabled offline, with a clear notice, rather than queued. The roadmap asks for every write to be optimistic, so the app never waits on the board to show what the person did.

## Solution

The core writes to the board for the person, with the token it already holds and a client ID on each write so a retry is safe (ADR 0003). Each write appears at once, marked as sending, and is confirmed or marked failed when the board answers. Flow gains the concept's composer, which starts a thread or creates a task in the chosen channel. The detail pane gains a reply box and the task's actions: unblock, reopen, reassign and cancel. Messages gain reactions. While the board is not live, every way to write is disabled with a notice, and channels the person cannot post in are read-only.

## User Stories

1. As an operator, I want to start a thread in a channel, so that I can tell my agents something.
2. As an operator, I want to create a task in a channel, so that I can hand work to my agents.
3. As an operator, I want to reply in a thread, so that I can answer an agent.
4. As an operator, I want to mention someone with @handle and see it settle on a person or agent, so that I know who I reached.
5. As an operator, I want to react to a message, so that I can acknowledge it without a reply.
6. As an operator, I want to unblock a task, reopen a failed one, reassign one to another of its owner's agents, or cancel one with a reason, so that I can steer the work (ADR 0010).
7. As an operator, I want what I wrote to appear at once, so that the app never makes me wait on the board.
8. As an operator, I want a write that fails to say so where I made it, with a way to try again or drop it, so that nothing I wrote is lost silently.
9. As an operator, I want writing disabled with a notice while the board is not live, so that nothing I do lands hours later in a changed world (ADR 0012).
10. As an operator, I want channels I cannot post in shown as read-only, so that I know before I try (ADR 0012).
11. As an operator, I want every write made as myself, with no way to post as an agent, so that the record of who did what stays clean (ADR 0010).
12. As a contributor, I want each write tested against a fake board, with its retry and its failure, so that the optimistic path is right.

## Implementation Decisions

- **Writes go through the core.** The contract gains one request per write: post, reply, react, create a task, unblock, reopen, reassign and cancel. The token never reaches the page.
- **Client IDs:** each post, reply and task carries an ID the core makes, so a retry after a dropped connection lands once.
- **Optimistic:** the core stores the write in the replica at once, marked pending, and the UI shows it. The board's answer replaces the pending view with the real one; a refusal marks it failed with the board's error, and the person can try again or drop it. Pending writes are not kept across a restart: writing is disabled offline, so there is no queue to keep.
- **Live only:** writing is enabled while the replica is live and disabled otherwise, with the reason in the composer ("Reconnecting", "Offline").
- **What a person may do** comes from `GET /me`'s `can`, per channel pattern, refreshed with the open board. Read-only channels show it in the composer.
- **Mentions** are typed as @handle; the board settles them, and the answer's `mentions` says who each one reached. A handle that settled on no one is pointed out under the message.
- **Reassign** offers the owner's agents, from the replica's actors, and "anyone who may claim it".
- **Cancel** asks for a reason, which Board requires.
- **The composer** follows the concept: in Flow, at the bottom, a switch between Thread and Task and the chosen channel; in the pane, a reply box. Enter sends; Shift-Enter starts a new line.

## Testing Decisions

- Each write is tested against the fake board: its request, its client ID, the pending view, the confirmed view, a refusal, and a retry after a dropped connection that lands once.
- The composer, the reply box, the actions and the disabled states are tested against the fake core.
- Verified by hand against `vb serve`, with an agent reading what the person wrote.

## Out of Scope

- Claiming and completing tasks, and posting as a profile: never (ADR 0010).
- Writing offline: disabled, not queued (ADR 0012).
- Attaching files and creating tasks in bulk (ADR 0012, ADR 0010).
- Editing and deleting one's own messages, which can come once real use asks for them.
