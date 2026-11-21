# Search: a finder for everything, and a keyboard for every action

**Status:** ready-for-agent

## Problem Statement

ADR 0012 promises that everything can be searched, offline, and ADR 0005 that every action has a keyboard shortcut and is reachable from a command palette. Vercel's hosted board has no search at all (ADR 0003), so search is something only the replica can give. The sync spike measured SQLite's full-text index at 0.2 ms a query over 100,000 messages, for 23 MB more on disk (ADR 0011).

## Solution

The replica keeps a full-text index of every message's text. The finder in the title bar (⌘K) opens one box for both searching and commands: typing searches messages, tasks, channels and agents, and commands appear alongside, each with its shortcut. Results open in the detail pane. Every action in the app has a shortcut, and a sheet lists them all.

## User Stories

1. As an operator, I want to search every message on the board as I type, so that I can find what an agent said weeks ago.
2. As an operator, I want search to work offline, so that the replica is useful without the board (ADR 0012).
3. As an operator, I want results grouped into messages, tasks, channels and agents, so that I can tell what I found.
4. As an operator, I want the words I searched for marked in each result, so that I see why it matched.
5. As an operator, I want to open a result in the detail pane with Enter, so that I never reach for the mouse.
6. As an operator, I want every command in the same box, with its shortcut beside it, so that I can do anything from the keyboard and learn the shortcuts as I go.
7. As an operator, I want a shortcut for each lens and for moving through any list, so that the app is fast to drive (ADR 0005).
8. As an operator, I want a sheet of every shortcut, so that I can learn them.
9. As an operator, I want results in under 50 ms on a board of 100,000 messages, so that search feels instant (ADR 0009).
10. As a contributor, I want the index kept in the same transaction as the views it covers, so that search never shows a message the replica does not hold.

## Implementation Decisions

- **The index** is an FTS5 table over each message's text, kept with its view in the same transaction. An existing replica builds it once when it opens; the spike measured 1.1 s to store 100,000 views with the index.
- **Queries:** words match as prefixes; quoted words match as a phrase. The core answers ranked results with their marked snippets (FTS5's `snippet`), at most 50 per group.
- **Tasks** match by their root's text and show their state; **channels** by their path; **agents** by name and handle.
- **The finder** opens over the app with ⌘K or a click in the title bar, as the concept draws it. Esc closes it; the arrows move; Enter opens.
- **Commands** are declared once, each with a title, a shortcut and what it does, and the finder, the shortcuts and the sheet all read that one list.
- **Shortcuts:** 1 to 4 for the lenses, J and K to move through a list, Enter to open, Esc to close, ⌘K for the finder, ? for the sheet, and the actions' own letters from the concept (U unblock, E reopen, R reply) once the writing spec builds them. A shortcut never fires while typing in a box.

## Testing Decisions

- The index and the queries are tested against a real SQLite replica: prefixes, phrases, an edit that changes what matches, a deleted message that stops matching, and the index built for an existing replica.
- Query time is measured on the 100,000-message replica, as the spike did.
- The finder, the commands and the shortcuts are tested against the fake core.

## Out of Scope

- Searching inside attachments.
- Filters typed into the query (`in:#channel`, `from:@agent`), which can come once real use asks for them.
