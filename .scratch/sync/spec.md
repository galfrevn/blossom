# Sync: a replica of the open board, copied whole and kept live

**Status:** ready-for-agent

## Problem Statement

The app knows which local board is open and who the operator is on it, and nothing else. Every lens the first release promises (Attention, Flow, Tasks and Fleet) reads from a replica of the board (ADR 0003), and ADR 0012 promises that everything can be read and searched offline. That replica does not exist yet. The sync spike settled how to build one for a local board (ADR 0011, `documentation/research/board.md`): copy the whole board, store it in SQLite through `rusqlite`, and follow the feed from where the copy started.

## Solution

When a local board opens, the core copies it whole into a replica on disk, follows its feed so the replica stays in step within a fraction of a second, and resumes where it left off when the app restarts. The status bar says what the replica is doing: copying, with how far along it is; live, with how many messages it holds; or reconnecting. When the board stops answering its feed, the app notices, which also closes the case ticket 05 of the scaffold left open.

## User Stories

1. As an operator, I want the app to copy the open board without my asking, so that I never wait on the board for something I already have.
2. As an operator, I want to see that a copy is under way and how far along it is, so that I know why a large board is not complete yet.
3. As an operator, I want a board of 100,000 messages copied in well under a minute, so that the first connection is not a chore.
4. As an operator, I want the replica to stay in step with the board as agents write, so that what I see is never more than a moment old.
5. As an operator, I want a message an agent posts to reach the replica in under a second, so that the app feels live.
6. As an operator, I want edits, deletions, reactions, label changes and task changes to reach the replica too, so that it matches the board and not just its new posts.
7. As an operator, I want an older view of a message never to replace a newer one, so that the replica is right whatever order the copy and the feed deliver things in.
8. As an operator, I want the replica to survive a restart of the app, so that reopening a board is instant and does not copy it again.
9. As an operator, I want the app to pick up the feed exactly where it left off after a restart or a dropped connection, so that nothing is missed and nothing is counted twice.
10. As an operator, I want the app to reconnect by itself when the board closes the feed, as `vb serve` does every few minutes, so that I never notice it happening.
11. As an operator, I want to see when the app is reconnecting, so that I know the replica may be a moment behind.
12. As an operator, I want a different board that runs at the same address as one I copied before to get a replica of its own, so that two projects' boards are never mixed.
13. As an operator, I want the app to notice a board that stopped without removing its token file, so that it stops showing a board that is gone.
14. As an operator, I want each board's replica kept on my Mac and nowhere else, so that the copy stays as private as the board (ADR 0013).
15. As an operator, I want the app to stay within its memory budget while it copies 100,000 messages, so that a large board does not make my Mac slow (ADR 0009).
16. As a contributor, I want the replica's behaviour tested against a real SQLite file in a temporary folder, so that the tests exercise the storage the app ships with.
17. As a contributor, I want the sync tested against a fake board that serves the real feed format, so that copying, following and reconnecting are covered without a running `vb serve`.
18. As a contributor, I want the token never to reach the replica's file or the page, so that a copied board carries no credentials.
19. As a contributor, I want the replica's queries to stay under a millisecond at 100,000 messages, so that the lenses built on it start from a fast base.

## Implementation Decisions

### The replica

- One SQLite file per board, in the app's data folder (ADR 0020's identifier), through `rusqlite` with SQLite bundled (ADR 0011). The folder is a parameter of the core, so tests point it at a temporary directory.
- The replica stores each message's latest view as the board sent it, with its channel, thread, creation time and `version` alongside, so that a view replaces the stored one only when its `version` is higher. Indexes by channel and by thread come with the first query that needs them.
- The replica also stores the feed's cursor, so that following resumes from it.
- **A board's identity is its address plus the creation time and id of its oldest message.** A local board has nothing else that says which project it belongs to, and its message IDs restart for every board. When a board at a known address shows a different oldest message, it gets a new replica.
- SQLite runs on a thread of its own. The sync hands it batches of views, one transaction per batch.
- The token stays in memory in the core. The replica holds no credentials.

### Copying

- A copy starts by reading the board's feed position, from the first page of `GET /threads`. The feed is followed from that cursor while the copy runs, and merging by `version` makes their order irrelevant.
- On a local board, the copy replays the feed from the oldest message, which is the fastest way the spike measured: 6.8 seconds for 100,000 messages. The oldest message is the root with the earliest creation time, found by paging through `GET /threads`. Cursors are never counted with.
- Progress comes from `GET /channels`, whose message counts a local board answers. Their sum is the copy's total.
- Showing the newest threads first, which ADR 0011 asks for, waits for the Flow lens, the first thing that shows them.

### Following

- The feed is followed over SSE, with `Last-Event-ID` set to the replica's cursor, so a reconnection resumes exactly where it left off.
- When the board closes the stream, about every 280 seconds, the core reconnects at once. When a connection fails, it retries with a growing pause and reports that it is reconnecting.
- When the feed keeps refusing connections and discovery stops answering, the board is gone: the open board becomes "none running", and the replica stays on disk.

### The contract

- A second subscription, the replica's state. It is copying (messages stored, and the total when the board says it), live (messages stored), or reconnecting (messages stored). It carries no message content: the lenses ask for that once they exist, and the list spike decides how (ADR 0018).
- The status bar shows the replica's state next to the board's address and the person: for example, "Copying 42,000 of 100,006", "100,006 messages", or "Reconnecting".

## Testing Decisions

- A good test drives the code through the seam a real caller uses and asserts what that caller sees.
- **The replica** is tested through its public interface against a real SQLite file in a temporary folder:
  - a newer view replaces an older one, and an older one never replaces a newer one;
  - the cursor is kept across reopening;
  - a different board at the same address gets its own replica;
  - the message count is right.
- **The sync** is tested against a fake board that serves the real formats: `/threads`, `/channels` and `/events`, both as JSON pages and as an SSE stream. A stream that ends after a few frames stands in for the board closing it, which exercises reconnecting with `Last-Event-ID`. The cases:
  - a whole copy;
  - an edit or deletion arriving during the copy;
  - resuming after a restart;
  - a cursor the board no longer accepts, which starts a new copy;
  - a board that stops answering.

  How the fake board streams SSE (a small server of its own, or a test dependency) is chosen in the first ticket that needs it.
- **The UI** is tested against the fake core: the status bar shows each of the replica's states.
- **By hand, against a real `vb serve`** holding 100,000 messages: the copy time, a post reaching the replica, and a restart that does not copy again.
- Prior art: the core's tests in `core/tests/` drive the open board with wiremock and temporary token folders; this follows the same shape.

## Edge Cases

Each case, and where it is handled. The tests named are in `core/tests/sync.rs`, `follow.rs`, `replica.rs` and `sse.rs`.

| Case | What the app does | Where |
| --- | --- | --- |
| The board closes the feed, as `vb serve` does every 280 seconds | Reopens it at once from the replica's cursor | 02, tested and measured |
| The board refuses the feed, or cannot be reached | Says "Reconnecting" and asks again after 1 s, doubling up to 30 s | 02, tested |
| The board ends every stream as soon as it opens it | Counts that as a refusal, so it is never asked again in a tight loop | 02, tested |
| A stream is cut inside a frame | Drops the partial frame; the next stream resumes from the last whole one | 02, tested |
| A stretch of the feed carries only bookkeeping | Still moves and stores the cursor | 02, tested |
| A page of the copy fails, or the board answers with an error | Says "Reconnecting" and asks for that page again, with the feed's pauses, instead of abandoning the copy | 02, tested |
| The board is slow to answer a page of the copy | Waits up to 45 s of silence; only discovery expects an answer within 2 s | 02, tested |
| An older view of a message arrives after a newer one | Keeps the newer one, by `version` | 01, tested |
| `vb serve` restarts and mints a new token | Opens the board again with the new token; until 03 that means copying it whole again | 02, tested and verified by hand |
| The board is killed and leaves its token file behind | Says "Reconnecting", then "No board running" about 3 s later; the replica stays | 04, tested and verified by hand |
| The board freezes, holding its connections without answering | 45 s of silence ends the stream, and opening another waits at most 10 s; "No board running" follows within about 90 s, and the board opens again once it answers | 04, tested and verified by hand |
| The board refuses its feed but still answers discovery | Keeps reconnecting; it is never called gone | 04, tested |
| The board sends an event or page the core cannot read | Stops, and says "Stopped: can't read the board"; restarting the app tries again | 04, tested |
| The replica cannot be opened or written | Stops, and says "Stopped: can't write the replica" | 04, tested |
| A different board starts at the same address, such as `vb serve` with another `--dir` on the same port | Its messages would join the first board's replica, which is named by address | 03, which names a replica by address and oldest message |
| The app restarts | Copies the board whole again | 03 resumes from the stored cursor |
| Two copies of the app share one replica, as a development build beside a release one did | SQLite keeps the file whole, but each app counts only what it stored itself | Left as is: a released app runs once per bundle |
| A post reaches the replica up to 2 s late | `vb serve` polls its own log, up to every 2 s when quiet; Blossom adds well under a millisecond | The board's, recorded in the research |
| A hosted board | Not opened: only loopback boards are found, so nothing reaches Vercel's API yet | Out of scope, below |

## Out of Scope

- Any lens, and any view of the board's content. The replica fills up and the status bar says so; Flow comes next.
- Showing the newest threads first (ADR 0011), which arrives with Flow.
- The full-text index, which arrives with search. The spike measured it at 0.2 ms a query.
- Reading a stopped board's replica offline. With no lens yet there is nothing to read, and it needs a decision on what the app opens on when the board is gone.
- Vercel's hosted board. The token spike (ADR 0022, proposed) settled how to get its token, not how to copy it. Every read there costs about five requests against Vercel's per-login rate limit, so a whole first copy of 100,000 messages would spend about 20,000 of the person's requests, an estimate not yet measured. The board also leaves out channel counts, so there is no total to show. Its first copy is decided once it is measured against a real hosted board.
- Writing, attachments and their downloads.
- Trimming what each stored view keeps. The spike stored full views: 98 MB for 100,000 messages.

## Further Notes

- The measurements behind these choices are in `documentation/research/board.md`, under "Copying a whole board, and following it".
- Board is experimental and moves fast. Check the feed's formats against the installed `vb` before building on them.
- `rusqlite` with SQLite bundled adds to the installer. The size is measured in the first ticket and checked against ADR 0009's 15 MB.
- The pipeline session runs CI and the hook. A change to the core's dependencies is coordinated with it, through the other session's messages.
