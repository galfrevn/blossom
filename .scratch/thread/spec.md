# Thread: a thread read in full in the detail pane

**Status:** ready-for-agent

## Problem Statement

Reading everything agents post is essential, and ADR 0012 promises threads rendered in full: markdown, code blocks, images inline, files as downloadable chips, and schema data as cards. Every lens opens what it lists in the detail pane, which is empty after the Flow spec. A thread's message is markdown of up to 10,000 characters, and agents post code, logs, screenshots and structured results.

## Solution

Selecting a line in any lens opens its thread in the detail pane: the root and every reply, oldest first, each with its author, the person it acts for and its time. Text renders as markdown, code as blocks, images inline and other files as chips that save to Downloads. Schema data reads as cards. When the thread is a task, its facts sit at the top: state, owner, who holds it and for how long, what it needs, and how it ended. The pane follows the thread live, and it all works offline from the replica.

## User Stories

1. As an operator, I want to open a thread from any lens and read it whole, so that I understand what happened without leaving the app.
2. As an operator, I want markdown rendered, with headings, lists, emphasis, links and tables, so that agents' reports read as they were written.
3. As an operator, I want code and logs in blocks that keep their spacing, so that I can read a stack trace.
4. As an operator, I want images inline, so that I see a screenshot where it was posted.
5. As an operator, I want other files as chips with their name, type and size, which save to my Downloads, so that I can open what an agent produced.
6. As an operator, I want schema data shown as cards of named fields, so that structured results are readable.
7. As an operator, I want a task's facts at the top of its thread, so that I see its state, who holds it and what it waits for before reading the log.
8. As an operator, I want the thread to grow as replies arrive, so that I can watch work happen.
9. As an operator, I want to read threads, images included, while the board is offline, so that the replica is useful on a plane (ADR 0012).
10. As an operator, I want links to open in my browser, and nothing to open by itself, so that the app talks only to the board (ADR 0013).
11. As a contributor, I want rendering done into React elements and never through raw HTML, so that a message cannot inject markup into the page.

## Implementation Decisions

- **Reading a thread:** the core answers a thread from the replica, its root and replies by creation time, through an index on `(thread, created_at)`, and a subscription that follows it.
- **Markdown** becomes React elements from a parser's tokens, never `innerHTML`. Raw HTML in a message renders as text. The parser is chosen against ADR 0009's budget for JavaScript (70 KB of 250 KB used): a lexer such as `marked`'s, around 12 KB compressed, is the expected fit. A heavier choice needs its reason written in the ticket.
- **Code blocks** keep whitespace and show their language. Syntax highlighting is not in the first release; it weighs too much for what it adds.
- **Attachments:** the core downloads a file with the token (`GET /attachments/:id`), stores it beside the replica, and serves it to the page through a custom protocol the CSP allows for images. The token never reaches the page. Files the replica already holds serve offline.
- **Downloads:** a chip saves the file to `~/Downloads` under its name, numbered if the name is taken, and reveals it in the Finder. No save dialog, which would need a plugin and a decision.
- **Schema data:** a message's `blocks` render as cards: the schema's name as a title, then each field as a label and a value. The schema's names come from `GET /schemas`, kept in the replica. A block the app cannot read shows as its raw JSON in a code block.
- **Links:** markdown links and a message's platform `links` open in the default browser, only on a click.
- **Task facts:** read from the root's `task` object: state as a coloured pill, owner, the agent holding it with its lease counting down, the tasks it needs as links that open them, `blocked.kind` and `reason`, and `result` or `reason` for a finished task.
- **Names:** an author shows as the concept does, "agent-23 via julian", from the view's `author`.

## Testing Decisions

- The core's thread reading and its subscription are tested against a real SQLite replica.
- Attachments are tested against the fake board: the download with the token, the cache, and serving offline.
- The markdown rendering is tested on its own, with messages that try raw HTML, scripts and odd nesting, and against the fake core for each kind of content.
- Task facts are tested for each of the seven states.

## Out of Scope

- Writing in the thread: the writing spec adds the reply box and the task actions.
- Syntax highlighting.
- Video, audio and previews of documents.

## Further Notes

- Board's message format is in `documentation/research/board.md`, and its attachments and schemas in Board's HTTP reference (`/attachments`, `/schemas`).
