# Hosted: a board on Vercel, read within the person's limits

**Status:** needs-info

## Problem Statement

Vercel hosts boards for teams, and Board's own cases run there. Blossom opens only local boards and boards on a tailnet (boards spec). A hosted board needs a token minted with the person's Vercel login, which ADR 0022 settles: Board's cache first, else the Vercel CLI, then the exchange at `oidc.vercel.com`. It also reads differently from a local board (`documentation/research/board.md`):

- It lists threads one channel at a time.
- It answers no channel counts.
- It keeps 30 days of feed.
- It times no lease.
- Every read through it costs about five requests against the person's per-login Vercel rate limit.

Copying a board of 100,000 messages whole would cost about 20,000 of those requests. That is an estimate nobody has measured, and the owner has no hosted board to measure it against.

## Solution

A pasted address whose discovery names a Vercel team and project is a hosted board. The core gets its token as ADR 0022 says, never touching the Vercel login itself. The first copy reads the newest threads of each channel first, then the rest in the background at a pace that leaves room in the person's rate limit. Progress shows what is stored, without a total. How much history to copy, and at what pace, is decided by measuring a real hosted board first.

## User Stories

1. As an operator on Vercel, I want to open my team's hosted board by pasting its address, so that Blossom works where my agents already run.
2. As an operator, I want Blossom to use the Vercel login I already have, so that I sign in nowhere new (ADR 0022).
3. As an operator, I want to be told to run `vercel login` when my login is missing or expired, so that I know how to fix it.
4. As an operator, I want the newest threads of every channel first, so that the app is useful within seconds.
5. As an operator, I want the rest of the history copied in the background without using up my Vercel rate limit, so that my other work on Vercel is not slowed.
6. As an operator, I want progress shown as what is stored, since the board gives no total, so that I still see it moving.
7. As an operator, I want a lapsed lease shown as lapsed, since the hosted board times none, so that a stalled agent shows (tasks spec).
8. As a contributor, I want the token flow tested against fake Vercel endpoints, so that it is covered without anyone's account.

## Implementation Decisions

- **Recognising a hosted board:** discovery's `team_id` and `project_id` are set; a local or tailnet board leaves them null.
- **The token** follows ADR 0022:
  - Read Board's cache in `~/.config/board/tokens/`, in Board's format, and use a token until five minutes before it expires.
  - Otherwise run `vercel api` and `vercel project token`, and exchange the result at `https://oidc.vercel.com/~token` for the board's audience.
  - Write it back to Board's cache.
  - The Vercel login is never read or written by Blossom, and the token never reaches the replica, the page or a log.
- **Reading** goes channel by channel, newest threads first, then each thread's replies, with the per-channel reads Board's hosted backend allows.
- **Pace:** reads stay well under the rate limit, and the core slows down when the board answers 429. The pace and how much history to copy are decided once measured (ticket 01).
- **The 30-day feed:** following resumes from the stored cursor; a replica older than the feed's reach is copied again.

## Testing Decisions

- The token flow is tested against a fake Vercel API, a fake OIDC endpoint and a fake `vercel` executable:
  - a cached token;
  - one about to expire;
  - a login that is missing;
  - an exchange that fails.
- The copy is tested against a fake hosted board that answers per channel, gives no counts, and answers 429.

## Out of Scope

- Creating a hosted board or deploying anything.
- Blossom's own "Sign in with Vercel" (ADR 0022's considered options).

## Open Questions

- **A hosted board to measure against.** The owner has none and has not used Vercel Board. The measurement is approved under limits: sample and extrapolate, stop at half the rate limit, no secrets written down. Creating a board for it needs its own decision.
