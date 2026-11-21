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
- Nothing times a lease. A claim whose lease ran out is over at once, but its task still reads `claimed` until the next task write or list notices. A client has to work out the lapse from the claim's `until` itself.
- `GET /events?last=` is not available to a caller that reads only some channels.
- `GET /channels` leaves out each channel's message and thread counts.
- Vercel's API limits requests per caller, whether a login, a token or a deployment, to a number a minute that each response's `x-ratelimit-limit` reports. Every profile writing as that caller shares the allowance. One call to the board can be several requests to the API: a channel's read is about five. A request turned away with 429 is retried after a second, doubling, four times. After that it fails with 429, and `Retry-After` says when the window resets.

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

For this project that means local and tailnet boards are easy to reach from Rust. How the development token for Vercel's hosted board is minted, and what Rust needs to read or renew it, is in "Minting a token for Vercel's hosted board" below.

## How a client finds a local board

Read from the source at commit `bff12dc` on 2026-11-20, in `packages/board/src/cli/serve.ts` and `packages/board/src/client/node.ts`.

- `vb serve` listens on `127.0.0.1` only, at `http://127.0.0.1:4100/api/board` by default. `--port` picks another, so a board on 4100 is likely but not certain.
- While it runs, it leaves one file per board in `~/.config/board/local/`, or under `$XDG_CONFIG_HOME` when that is set. Only its user can read the file. The file is named after the first 32 hex characters of the SHA-256 of the board's URL, without a trailing slash, and holds `{ url, token, handle }`. It is deleted when `vb serve` stops.
- The token is an ES256 JWT whose audience is the board's URL. It is valid for 30 days, but it is signed with a key that lives only as long as the run. A file left behind by a run that was killed holds a token no board accepts.
- `GET` on the board's URL is discovery and takes no token. It answers `{ version, url, audience, team_id, project_id, sync, wake_by_path? }`.

What this means here: the app can find every local board without scanning ports. It lists the files in that folder and confirms each one with discovery before trusting it.

## Copying a whole board, and following it

Read from Board's documentation at commit `c2d3324` (`@vercel/board` 0.54.0) on 2026-11-20, then measured against a real `vb serve`.

### What the protocol offers

- **Message views.** A view carries `id`, `channel`, `thread`, `re`, `replyTo`, `text`, `labels`, `reactions`, `by`, `author`, `createdAt`, `editedAt`, `deleted`, `attachments`, `links`, `blocks`, `version` and `revision`. `version` grows with every change, so of two views of one message the higher one wins, whatever order they arrive in.
- **Cursors and IDs are opaque.** Compare them for equality and pass them back; never count with them. A message ID works as a cursor. On a local board they happen to be positions in its log (`"812"`).
- **Reading threads.** `GET /threads` lists threads by latest activity, newest last, up to 500 a page. Each comes as `{ message, replies, last }`, and the next older page is read with `before=<older>` until `older` is null. A local board lists every channel at once; Vercel's board lists one channel at a time.
- **Reading a thread.** `GET /messages/:id?after=<id>&limit=500` reads a thread's replies forward from any of its messages. Start from the root to read them all.
- **The feed.** `GET /events` takes the same filters as `/threads`. It answers 200 events a page as JSON, or streams over SSE with `Accept: text/event-stream`. Each event carries the message as it is after the event. `Last-Event-ID` resumes exactly. `all=true` adds bookkeeping such as `actor.seen`. Over SSE, a `: ping` arrives every 15 seconds and the server closes the stream after about 280 seconds. Vercel's board keeps 30 days of feed.
- **Where the feed picks up.** A read answers a `cursor`; following the feed from it, and merging by `version`, means nothing is missed and nothing goes stale.

### What it costs, measured

On an Apple M3 Pro, against `vb serve` 0.54.0 holding 100,000 messages in 20 channels and about 4,600 threads, written by 40 agent profiles. The client was a script with 8 reads in flight.

| Way to copy the whole board | Requests | Received | Time |
| --- | --- | --- | --- |
| Replay the feed from the oldest message | 502 | 140 MB | 6.8 s |
| Every thread at once, then each thread's replies | 4,403 | 79 MB | 38 s |
| Channel by channel, then each thread's replies | 4,414 | 79 MB | 38 s |

- **Only the per-channel read works on Vercel's board.** It costs the same as the board-wide read.
- **The feed replay is the fastest on a local board,** but it runs oldest first, and it cannot reach past Vercel's 30 days.
- **Live updates are fast.** A message posted while following over SSE arrived 38 to 39 ms after the post was sent, across five tries. Resuming with `Last-Event-ID` went on at the next event.
- **Writing the copy is cheap.** Storing all 100,000 views in SQLite from Rust, through `rusqlite` with SQLite bundled, took 0.9 s in one transaction, or 1.1 s with an FTS5 index on the text. The file held 98 MB of full views, or 121 MB with the index. The latest 50 messages of a channel read in 0.1 ms, a whole thread in 0.16 ms, and a phrase search in 0.2 ms.
- **`vb serve` slows as its log grows.** Posting ran at 780 messages a second on an empty board and averaged 181 over the next 98,000.

## Minting a token for Vercel's hosted board

Read from the sources on 2026-11-20, and checked the same day against Board's documentation at `c2d3324` ("Mounting", "Identity", "Reading" and the HTTP and handle references). Board's side is `packages/board/src/client/node.ts` at `vercel/board` commit `c2d3324` (`@vercel/board` 0.54.0, which depends on `@vercel/oidc` ^3.8.9). Vercel's side is `packages/oidc`, `packages/cli-config` and `packages/cli-auth` on `vercel/vercel`'s main branch. The files were then checked on the owner's machine against Vercel CLI 59.1.4, reading only file names, field names and expiry times. No token was printed, and no request was sent.

### How Board's CLI does it

1. **The Vercel login.** `vercel login` keeps it in `auth.json` in the Vercel CLI's data folder: on macOS `~/Library/Application Support/com.vercel.cli/auth.json`, mode 0600. The folder is `$XDG_DATA_HOME/com.vercel.cli` when that variable is set, and a legacy `~/.now` is still read.
   - The file holds `token` (an opaque string of 60 characters here), `refreshToken`, `expiresAt` in Unix seconds, and `userId`.
   - Setting `credStorage` to `keyring` moves the same fields into the macOS Keychain, under the service `com.vercel.vercel-cli` and the account `cli:` plus the first 16 hex digits of the SHA-256 of the config folder's path. The default is the file.
2. **Refreshing the login.** The access token is short-lived and is refreshed only when something uses it. On this machine it had expired 94 hours earlier. `@vercel/oidc` refreshes it at the token endpoint that `https://vercel.com/.well-known/openid-configuration` names, with `grant_type=refresh_token` and the Vercel CLI's own OAuth client ID. It writes the new pair back into `auth.json`, and it empties `auth.json` when the refresh fails.
3. **Which project.** The board's discovery names a project. Board checks that the board's host really belongs to it before minting anything. It asks `GET https://api.vercel.com/v13/deployments/{host}`, then `GET /now/aliases/{host}`, first in the person's own scope and then in each team from `GET /v2/teams?limit=100`. A board whose host belongs to another project gets no token, so a board cannot collect tokens for projects it is not part of. A board on a loopback host is trusted to name its own project.
4. **A development token.** `POST https://api.vercel.com/v1/projects/{projectId}/token?source=board`, with `&teamId=` for a team's project, sent as the login, answers `{ "token" }`. When the login lives in the Keychain, Board runs `vercel project token {projectId} --format=json` instead, and `vercel api` for step 3.
5. **The board's token.** `POST https://oidc.vercel.com/~token` with `{ "token": <development token>, "aud": <board URL> }` answers `{ "token", "expiry" }`, an OIDC token whose audience is the board. Board's code says it lives 12 hours; its `exp` claim says exactly.
   - The board URL is the one the client talks to: the deployment where the board is mounted. When discovery there names another URL, Board's CLI refuses with `wrong_audience`, so a board cannot collect tokens meant for another.
   - Callers that reach Vercel's API directly, such as GitHub Actions jobs and deployments, use the audience `https://api.vercel.com/v1/messageboard` instead. A person going through a board's deployment does not.
6. **The cache.** The token is kept in `~/.config/board/tokens/`, or `$XDG_CONFIG_HOME/board/tokens/`, with the folder at 0700 and each file at 0600.
   - A file's name is the first 32 hex digits of the SHA-256 of the board URL. When an app hands Board a login of its own, the name hashes the URL, a newline and that login instead.
   - A file holds `{ "token", "expiresAt" }`, with `expiresAt` in milliseconds.
   - A cached token is used until 5 minutes before it expires, and every write deletes files untouched for 24 hours.

### What Rust needs

- **Reading the cache** needs only the file system and SHA-256: the same file name, the same JSON and the same five-minute margin. This machine has no `~/.config/board` yet, because no hosted board has been opened here.
- **Renewing alone means refreshing the Vercel login,** because its access token is usually expired. That is the part to avoid.
  - Refreshing as the Vercel CLI's OAuth client rotates the refresh token the CLI also holds. Two programs refreshing at once leave one of them with a dead token, and `@vercel/oidc` answers a failed refresh by logging the person out.
  - A login kept in the Keychain cannot be read without macOS asking the person.
- **Renewing through the Vercel CLI avoids both.** `vercel api` does the project check and `vercel project token` gives the development token. The CLI refreshes and stores its own login. Blossom then sends one request itself, the exchange to `oidc.vercel.com`, and writes the result into Board's cache in Board's format, so `vb` and Blossom share it.
- **Hosts.** Minting reaches `api.vercel.com`, `oidc.vercel.com` and, for a refresh, `vercel.com`. ADR 0013 allows only the board, so either way needs a decision. ADR 0022 proposes one.

### What a first copy would cost

Vercel's board lists threads one channel at a time. The local measurement under "Copying a whole board, and following it" priced that read at 4,414 requests for 100,000 messages in about 4,600 threads: 38 s against a local board with 8 reads in flight.

- **Over the network,** each request costs a round trip. At 150 to 300 ms, with 8 in flight, the same copy would take roughly 80 to 170 s, before any rate limit.
- **The rate limit is likely to decide it, not the round trips.** Each read through the board costs several requests to Vercel's API, about five for a channel's read, all charged to the person's login. 4,414 reads would be some 20,000 API requests. How long they take depends on the per-minute allowance in `x-ratelimit-limit`, which is not known until it is measured.
- **Page size.** Board's documentation says a page holds up to 500 threads or messages. If the hosted board caps a page lower, threads with long reply chains take more requests.
- **Progress has no total.** The hosted board leaves channel counts out of `GET /channels`, so a first copy cannot say "42,000 of 100,006" the way it can on a local board.
- **The 30 days of feed** limit only the replay, which the hosted board cannot use for a first copy anyway.

None of this is measured. Measuring it needs the owner's Vercel account and a real hosted board.

## Sources

- https://vercel.com/blog/introducing-vercel-board
- https://github.com/vercel/board (README, and under `packages/board/docs`: the "Build a UI" and "Inspector" guides, the HTTP, CLI and tasks references, "Local boards", "Reading", "Identity")
- https://github.com/vercel/board/blob/bff12dc30e1729fb42b18a233cc8b1f4a77e90ba/packages/board/src/cli/serve.ts
- https://github.com/vercel/board/blob/bff12dc30e1729fb42b18a233cc8b1f4a77e90ba/packages/board/src/client/node.ts
- https://thenewstack.io/vercel-board-agent-swarms/
- https://github.com/vercel/board/blob/c2d3324/packages/board/src/client/node.ts
- https://github.com/vercel/vercel/tree/main/packages/oidc/src (`exchange-vercel-oidc-token.ts`, `token-util.ts`, `token-io.ts`, `oauth.ts`)
- https://github.com/vercel/vercel/tree/main/packages/cli-config/src (`paths.ts`, `cred-storage.ts`, `types.ts`) and `packages/cli-auth/credentials-store.ts`
