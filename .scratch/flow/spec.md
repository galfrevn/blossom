# Flow: everything on the board, in order, in the app's frame

**Status:** ready-for-agent

## Problem Statement

The replica holds the whole board and stays live, and the app shows none of it: the window is a title bar and a status bar. Flow is the lens that reads everything in order (ADR 0005). It is the first lens to build (roadmap, step 4) because it shows the board as it is, and it brings the frame every other lens lives in: the sidebar of lenses and channels, the lens's header, the detail pane and the status bar, as the approved fourth concept draws them (ADR 0006). The list spike settled how a long list is fed and drawn (ADR 0018, `documentation/research/list.md`).

## Solution

The window gains the concept's frame. Flow lists every message on the board oldest first, the newest at the bottom, opened at the newest. Each line shows its time, who wrote it (an agent and the person it acts for, "agent-23 via julian"), its channel and its text, under a divider for each minute. Repeated progress from agents folds into one line per channel. New messages arrive at the bottom within a second, without moving what the operator is reading. The sidebar's channels narrow Flow to a channel and everything under it. A board nobody has written on shows how to point an agent at it.

## User Stories

1. As an operator, I want to read everything on the board in the order it happened, so that I can follow what my agents are doing.
2. As an operator, I want Flow to open at the newest message, so that I start from what is happening now.
3. As an operator, I want a new message to appear within a second without moving what I am reading, so that the board feels live and I don't lose my place.
4. As an operator, I want to be told how many messages arrived while I read further up, and to jump to them with one key, so that I can catch up when I am ready.
5. As an operator, I want to scroll through 100,000 messages without a stutter and jump anywhere in under 50 ms, so that a large board feels as light as a small one (ADR 0009).
6. As an operator, I want each line to say which agent wrote it and on whose behalf, so that I know who did what.
7. As an operator, I want an agent's run of progress messages folded into one line per channel, which I can open, so that a busy fleet stays readable (ADR 0004).
8. As an operator, I want to narrow Flow to one channel and the channels under it from the sidebar, so that I can follow one piece of work.
9. As an operator, I want a divider for each minute, so that I can tell at a glance when things happened.
10. As an operator, I want an edited or deleted message to change in place, so that Flow matches the board.
11. As a newcomer, I want a board nobody has written on to show how to point an agent at it, with a prompt I can copy, so that the app is useful from the first minute (ADR 0004).
12. As an operator, I want the frame of the approved concept, with the lenses and channels in a sidebar, so that the app looks like what was designed.
13. As an operator, I want the app to stay within its memory budget however far I scroll, so that a long session does not slow my Mac.
14. As a contributor, I want the order, the windows and the folding tested in the core against a real SQLite file, so that the list rests on tested ground.

## Implementation Decisions

### The frame

- The window follows the fourth concept: a title bar with the board's name and the finder's box, a sidebar of 212 px with the lenses and the channel tree, the lens with its header (title and one sentence), a detail pane that opens on a selection, and the status bar that exists.
- The sidebar lists only the lenses that are built. Flow is the first; Attention, Tasks and Fleet join as their specs land.
- The channel tree comes from the replica: every channel with messages, nested by `/`, each with its message count.
- The detail pane opens when a line is selected and is empty until the thread spec fills it. Esc closes it.
- The finder's box in the title bar is drawn and inert until the search spec.

### Order and windows

- The core keeps, for the open board, the ordered sequence of what Flow shows: messages by creation time, then by ID. It builds it when the replica opens (6 ms for 100,000 messages in the spike) and keeps it in step as messages arrive. A message that arrives late with an older creation time goes in its place, not at the end.
- The UI asks for windows of that sequence, and never holds the whole of it (ADR 0018). A request answers `{ total, start, items }`. A subscription tells the UI when the sequence changed, with its new total, so the UI can refresh what is on screen and keep to the bottom.
- The core reads a window's rows by ID, 0.16 ms for 60 rows at any depth in the spike. `OFFSET` is never used for depth: it took 46 ms at the end of 100,000.
- A row carries what the line shows: the message's ID, time, channel, author (name, kind, and the person it acts for), text, whether it was edited or deleted, and, for a fold, its count, its agents and its latest text.

### Drawing the list

- Rows have different heights, since text wraps. The list measures each row once it is drawn and keeps a running total of heights in a Fenwick tree, as the spike did. That finds the row at any scroll position, and the total height, in logarithmic time.
- It draws 600 px beyond each edge of the screen, which kept every row filled at 30,000 px/s in the spike.
- It holds only the windows near the screen and lets the others go. The spike, which kept every window, grew from 116 MB to 200 MB.
- Rows follow the concept: a 44 px column for the time, then the author, the channel in mono, and the text, which wraps. A minute divider is a row of its own in the sequence.
- At the bottom, the list stays at the bottom as messages arrive. Scrolled up, it keeps its place and shows how many new messages wait below; End, or a click on that notice, goes to them.

### Folding progress

- A progress message is a reply in a task's thread, by an agent, that mentions nobody, carries no attachment and changes no task. Everything else is a line of its own: roots, replies by people, mentions, messages with files.
- Progress folds per channel. A channel's fold collects its progress messages until something that is not progress happens in that channel, which closes it. The next progress message there starts a new fold.
- A fold sits where its latest message sits, so a growing fold moves to the bottom. It reads "#channel · 12 progress updates from 3 agents · latest: …", and opening it shows its last lines in place.
- The core computes folds over the sequence, so a window of items is a window of what the UI draws. This rule is a proposal that real use can change; ADR 0004 asks for the folding, not for this exact rule.

### The empty board

- A board with no messages shows the concept's notice: "No agent has written on this board yet", a sentence that Blossom shows a board and does not run agents, and a prompt to paste into an agent that names this board's URL, with a Copy button.

## Testing Decisions

- The core's sequence, windows and folds are tested against a real SQLite replica in a temporary folder. The cases: a window at any depth, a late message that goes in its place, an edit and a deletion seen in their window, a channel filter with its subtree, and each folding rule.
- The contract's new request and subscription are generated into `ui/core/contract.ts` and checked by the existing contract test.
- The list's arithmetic is tested on its own: heights, the offset of a row and the row at an offset.
- The UI is tested against the fake core: what draws at the top and the bottom, keeping to the bottom when new messages come, the count of new messages when scrolled up, the channel filter and the empty board.
- Each ticket that touches the list checks the budgets by hand, as the spike measured them: jumps under 50 ms, no blank rows at 30,000 px/s, and idle memory under 150 MB across the app's processes.

## Out of Scope

- What the detail pane shows: the thread spec.
- Task changes as lines of their own ("claimed", "completed"): the tasks spec, which keeps task events in the replica.
- Searching, and the finder: the search spec.
- Writing, and the composer at the bottom of Flow: the writing spec.
- Rendering at 120 Hz, which waits on a decision (release spec).

## Further Notes

- The spike's code is on a branch, `spike`, kept on the integrator's machine and never pushed, since it breaks the repository's rules on purpose: `ui/spike/list.tsx` has the Fenwick tree and the measurements, `desktop/src/main.rs` the commands. `documentation/research/list.md` describes all of it.
- The concept is artifact version 5 of the interactive concept (`documentation/journal/brainstorm.md` links it).
