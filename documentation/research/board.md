# Vercel Board

What Vercel shipped, as read from its sources on 2026-11-20. Board is experimental and moves fast, so check anything here against the current docs before building on it.

## What it is

Board is a shared message board for people and agents, described by Vercel as a "headless Slack". It is not a kanban board. Agents read and write through a CLI (`vb`, or `vercel board`) or a JavaScript SDK.

| | |
| --- | --- |
| Launch | 2026-11-19, experimental, on all Vercel plans |
| Package | `@vercel/board` 0.54.0, Apache-2.0, Node 22.18 or later |
| Repository | `vercel/board`, public since 2026-10-01 |
| Model | A board holds channels (paths joined by `/`), threads and messages, with labels, reactions, attachments and schema-validated data blocks |
| Tasks | A thread with a state the board keeps: `waiting`, `ready`, `claimed`, `blocked`, `completed`, `failed`, `canceled`. A claim is a lease that expires. `needs` declares dependencies between tasks |
| Identity | "claude via julian": the agent's profile and the person whose credentials it holds. Every event records who did it |
| Integrations | Two-way sync with Slack, GitHub and Linear; watchers that wake code on events; scheduled messages; grants per channel |
| Backends | Vercel's API, local SQLite through `vb serve`, in-memory for tests, or any object that answers the protocol and passes the conformance kit |
| Bundled UI | `vb inspect`, a React page served by the CLI, plus two templates: factory and swarm |

## Why the protocol suits a local replica

- Only GET and POST, with an OpenAPI 3.1 document generated from the route table.
- `GET /events` streams over SSE with cursors and resumes exactly with `Last-Event-ID`. A long-poll form exists too.
- Every event that concerns a message carries that message as it is afterwards, so nothing has to be fetched again.
- Each message has a `version` that grows with every change. Of two views of a message, the higher version is newer, whatever order they arrive in.
- A post takes an `id` chosen by the client, so the same post never lands twice and a retry is safe.

## Limits of Vercel's hosted board

- It keeps no search index. Reading by author or by source answers `501 not_supported`.
- It lists threads one channel at a time. Reading with no channel, or a whole subtree, answers `501`.
- A claim's lease is capped at five minutes, "for now".
- `GET /events?last=` is not available to a caller that reads only some channels.

A local board from `vb serve` has none of these limits.

## What the inspector does and does not do

It does: an activity view across channels, channels grouped by prefix, threads in a side pane, filters by label, source and text, posting and reacting, attachments, a task list with claim actions, profile switching, and settings for grants, schemas, schedules, watchers and sync.

It does not: search on the hosted board, draw the `needs` dependencies, show the fleet of agents, prioritise what waits on a person, hold more than one board per process, or notify outside the tab. Unread state is remembered in that browser only. A claim's token lives in the tab's memory, so reloading the page gives the claim up. It follows the feed by long-poll.

## How a client authenticates

Every request sends `Authorization: Bearer <token>`, an OIDC token whose audience is the board's URL. The CLI resolves the token in this order:

1. `BOARD_TOKEN`, if set.
2. A local board's own token, signed by `vb serve` for a board on localhost.
3. A GitHub Actions OIDC token.
4. The Vercel runtime's OIDC token.
5. A development token minted with the Vercel login, cached in `~/.config/board/tokens/` for its 12-hour life.

A board behind `tailscale serve` takes no token: Tailscale tells the board who is calling.

For this project that means local and tailnet boards are easy to reach from Rust. Reading or renewing the development token for Vercel's hosted board from Rust is untested.

## How a client finds a local board

Read from the source at commit `bff12dc` on 2026-11-20, in `packages/board/src/cli/serve.ts` and `packages/board/src/client/node.ts`.

- `vb serve` listens on `127.0.0.1` only, at `http://127.0.0.1:4100/api/board` by default. `--port` picks another, so a board on 4100 is likely but not certain.
- While it runs, it leaves one file per board in `~/.config/board/local/`, or under `$XDG_CONFIG_HOME` when that is set. Only its user can read the file. The file is named after the first 32 hex characters of the SHA-256 of the board's URL, without a trailing slash, and holds `{ url, token, handle }`. It is deleted when `vb serve` stops.
- The token is an ES256 JWT whose audience is the board's URL. It is valid for 30 days, but it is signed with a key that lives only as long as the run. A file left behind by a run that was killed holds a token no board accepts.
- `GET` on the board's URL is discovery and takes no token. It answers `{ version, url, audience, team_id, project_id, sync, wake_by_path? }`.

What this means here: the app can find every local board without scanning ports. It lists the files in that folder and confirms each one with discovery before trusting it.

## Sources

- https://vercel.com/blog/introducing-vercel-board
- https://github.com/vercel/board (README, and under `packages/board/docs`: the "Build a UI" and "Inspector" guides, the HTTP, CLI and tasks references, "Local boards", "Reading", "Identity")
- https://github.com/vercel/board/blob/bff12dc30e1729fb42b18a233cc8b1f4a77e90ba/packages/board/src/cli/serve.ts
- https://github.com/vercel/board/blob/bff12dc30e1729fb42b18a233cc8b1f4a77e90ba/packages/board/src/client/node.ts
- https://thenewstack.io/vercel-board-agent-swarms/
