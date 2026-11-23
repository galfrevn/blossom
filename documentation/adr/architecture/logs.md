---
adr: 0024
status: accepted
date: 2026-11-23
---

# The replica trims its logs of changes once the reader has read them

The replica keeps three logs, so that the open reader can catch up with what changed without reading the board again (ADR 0003):

- `arrivals`, a row for every message stored (Flow 03);
- `task_changes`, a row for every insert and update of a task (Tasks 02);
- `message_changes`, a row for every change to a stored message's facts, and one for each reply already in a thread when it becomes a task (Flow 08).

None of them was ever trimmed. On the 100,000-message board `arrivals` holds a row per message, `task_changes` about 1.6 rows per task event, and `message_changes` took 93,951 rows in one batch when every thread became a task (Flow 08's comments). Since Sync 05, every renewal of a claim is a task event, so `task_changes` grows for as long as an agent holds a task.

The reader needs only the rows after its cursor. It lives in the same process as the writer, and a reader that opens builds what it needs from the tables, not from the logs. So **the writer deletes the rows of `task_changes` and `message_changes` that every open reader has already read.** The owner chose this on 2026-11-23.

**`arrivals` is kept whole.** Its sequence is also each message's rowid in the full-text index (Search 01), because `messages` has no rowid of its own. Each of its rows holds only a message's ID.

## Consequences

- A reader tells the writer how far it has read in each log. The writer deletes up to the lowest such point among the open readers, and nothing while no reader is open.
- A reader can never miss a change it has not read: the writer keeps every row after the lowest point read.
- The two logs then hold what the readers have not read yet, usually nothing, however old the board is.
- Trimming `arrivals` would first need the full-text index keyed by something else. That is a decision for another ADR, if the file's size ever calls for it.

## Considered options

- **Keep every row.** The simplest, and small on today's boards, but `task_changes` grows with every renewal for as long as the replica lives.
- **Trim all three logs, keying the full-text index by a rowid of its own.** It saves little, since a row of `arrivals` holds only an ID, and it would rebuild the index on every replica.
