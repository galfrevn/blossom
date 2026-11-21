# Tasks: every task on the board, and what happens to each

**Status:** ready-for-agent

## Problem Statement

Tasks are how work is handed to agents, and a fleet runs thousands of them. The replica keeps each message's latest view, which carries a task's current state on its root, and drops the feed's events. It cannot say when a task changed, who claimed it, or that a lease ran out. The Tasks lens (ADR 0012), Flow's lines for task changes, Fleet's lapses and Attention's failures all need that history.

## Solution

The replica keeps the feed's task events and the actors behind them, and an index of tasks built from their roots. The Tasks lens lists every task on the board, open ones first, with a chip per state and its count, a filter by title, and the concept's columns: state, task, channel, owner, held by with its lease, and when it last changed. It stays fast with tens of thousands of tasks. Flow gains a line for each task change: claimed, released, blocked, completed, failed, canceled, reopened.

## User Stories

1. As an operator, I want every task on the board in one list, open ones first, so that I see the state of the work.
2. As an operator, I want a chip per state with its count, which narrows the list, so that I can see only what is blocked or failed.
3. As an operator, I want to filter tasks by their title, so that I find one quickly.
4. As an operator, I want to see who owns a task and which agent holds it, with its lease counting down, so that I know who is on it and for how long.
5. As an operator, I want a lapsed lease shown as lapsed, even while the board still says claimed, so that a stalled agent does not hide (Board's hosted backend times no lease).
6. As an operator, I want a waiting task to say what it needs, so that I can see the chain of work.
7. As an operator, I want to see when each task last changed, so that I can spot work that has stopped.
8. As an operator, I want task changes as lines in Flow, so that the board's story includes who claimed what and how it ended.
9. As an operator, I want the list to scroll and narrow instantly with tens of thousands of tasks, so that a large fleet is as easy as a small one (ADR 0004).
10. As an operator, I want to open a task's thread from the list, so that I can read its log and facts.
11. As a contributor, I want the task events and the task index tested against a real SQLite replica and a fake board, so that the history is right.

## Implementation Decisions

### What the replica keeps

- **Task events.** As the copy and the feed deliver events, the replica keeps each `task.*` event: its cursor, type, the task's ID, the actor, its `createdAt`, and the fields that matter (`reason`, `result`, `for`, `needs`). On a local board, the copy replays the feed from the oldest message, so a fresh copy has the whole history.
- **Actors.** Every event names its actor (`id`, `kind`, `handle`, `name`, `via`). The replica keeps the latest of each by ID, so owners and holders show by name. An ID never seen in an event is asked for with `GET /actors?id=`.
- **A task index**, one row per task, from the root's `task` object: state, owner, the actor it was made as, who it is for, who holds it and until when, what it needs, the block's kind and reason, the result or reason, and the time of its latest event. It is kept in the same transaction as the view that changed it.
- **An older replica is copied again.** The replica gains a schema version, and one made before this spec has no task events. No release has shipped, so there is nothing to migrate.

### The lens

- **Order:** open tasks first, in the order blocked, failed, claimed, ready, waiting, then completed and canceled; within a state, the latest change first. This follows the concept's order; real use can change it.
- **Windows**, as Flow's: the UI asks the core for windows of the ordered, narrowed list. Rows are a fixed 30 px, as the concept draws them, so no measuring is needed.
- **The chips' counts** come from the index, narrowed by the channel chosen in the sidebar.
- **The title filter** narrows by the root's first line, case-insensitively. Full-text search is the search spec's.
- **Leases** count down from `claim.until` in the UI, without asking the core every second, and read as lapsed once past.

### Flow's lines for task changes

- Each task event the replica keeps becomes a line in Flow's sequence: "claude via julian claimed fix CVE-123", with the state's pill. `task.renewed` is bookkeeping and makes no line.

## Testing Decisions

- The task events, the actors and the index are tested against the fake board, whose history gains task events: a task created, claimed, renewed, blocked, unblocked, failed, reopened, completed, canceled and expired. They are also tested across a copy and a live follow.
- The order, the chips' counts and the filters are tested in the core.
- The lens is tested against the fake core: rows, chips, the filter, leases counting down and a lapsed lease.
- Flow's task lines are tested in the core's sequence.

## Out of Scope

- Acting on a task (unblock, reopen, reassign, cancel): the writing spec.
- Creating tasks: the writing spec.
- The graph of what waits for what: Graph comes after the first release (ADR 0012).
