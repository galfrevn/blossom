# Pipeline: CI that answers in under two minutes, and releases on a tag

**Status:** ready-for-agent

## Problem Statement

Every push to `main` waits more than six minutes for its result, and the wait is not the checks: the Rust cache has never hit. All six jobs of the first three runs started cold, and re-running a commit with its cache saved still missed on macOS, because the cache key changes whenever GitHub rolls out a new runner image. The macOS job is the critical path, and it compiles Tauri from scratch in release mode on three cores every time.

The project has no way to publish a build either. The app is built locally, and nothing turns a version into something a person can download.

## Solution

Make the cache hit, split the macOS job in two so its lint and its installer build run side by side, and keep every check honest: the full suite on every push, no retries, no skipped tests, no timeouts widened to pass. A pre-push hook becomes the gate before `main`, since there is no pull request in between. Each job gets a time budget that fails it when exceeded. A tag builds the disk image and publishes it on GitHub Releases, with checksums and a provenance attestation.

Everything stays free: GitHub's standard runners cost nothing on a public repository, and every tool added is open source.

## What was measured

All runs are pushes to `main` on 2026-11-20, on GitHub's standard runners (`ubuntu-latest`, and `macos-latest`, which is `macos-26-arm64`, three M1 cores). The history holds five runs, not the 150 an audit would want, so the figures are ranges, not percentiles.

| Run | Commit | Rust cache | Linux job | macOS job | Push to result |
| --- | --- | --- | --- | --- | --- |
| 44824643655 | `c4cb428` | miss | 74 s | 363 s | 6 min 07 s |
| 44826732380 | `9b2f6d1` | miss | 73 s | 380 s | 6 min 25 s |
| 44829533423, attempt 1 | `a0016ed` | miss | 94 s | 377 s | 6 min 21 s |
| 44829533423, attempt 2 | `a0016ed` | Linux hit; macOS miss, new runner image | 37 s | cancelled by the next push | none |
| 44838946020 | `e864dab` | partial: `Cargo.lock` changed | 39 s | 129 s | 2 min 29 s |
| 44840700119 | `6a70891` | full match on both | 36 s | 89 s | 1 min 32 s |
| 44840781257 | `e3c4f7e` | full match on both | 29 s | 105 s | 1 min 47 s |

The last two were measured after the plan was written. They are the warm baseline that tickets 01 and 02 measure against.

Split into waiting, setup and running:

| Job | Waiting | Setup | Running | Saving the cache |
| --- | --- | --- | --- | --- |
| Linux, cold | 1–3 s | 15 s | 50–70 s | 8–9 s |
| Linux, warm | 1–3 s | 24 s, of which the toolchain 10 s and the cache 7 s | 8 s | 0–5 s |
| macOS, cold | 2–3 s | 20 s | 4 min 45 s – 5 min 10 s: Clippy 78–98 s, the installer 3 min 18 s – 3 min 35 s | 37–54 s |
| macOS, partial | 2 s | 27 s, of which the toolchain 8 s and the cache 9 s | 70 s: Clippy 12 s, release compile 33 s, disk image 22 s | 26 s |

What the numbers say:

- **The biggest bucket is a cold cache, and on macOS it is a bug.** `Swatinem/rust-cache` hashes the Rust toolchains installed on the runner into its key. Image `20261102` shipped Rust 1.99.1 beside the pinned 1.100.0, and image `20261117` does not, so the same commit produced two keys: `688f0a87` and `60948658`. During every image rollout, macOS jobs alternate between keys and miss.
- **Waiting is 1–3 seconds.** Faster or paid runners would not shorten it. Run 44838946020 also waited 18 seconds for the run it replaced to cancel.
- **Lint, format and types take under a tenth of a second** locally: Biome 40 ms, `tsc` 7 90 ms, `bun check` 70 ms. The UI's tests take 1 s and the core's 2 s warm. They are not where the time goes.
- **Warm, the critical path is the installer**: 33 s of release compile and 22 s of disk image on three cores. That is the floor on free runners for an honest installer budget.
- **15 of the 21 commits on `main` touch only documentation.** They still deserve the full suite, for the reason under "Decisions".

## Decisions

Taken with the owner on 2026-11-20:

- **Pushes go straight to `main`.** One person works on the repository, so there is no pull request gate and no merge queue. Pull requests from Dependabot or outside contributors run the same full suite.
- **Everything is free.** GitHub's standard runners only. No paid runners, no organization, no paid accounts.

Recommended in this plan, and recorded in ADR 0021 by ticket 01:

- **The full suite runs on every push, with no selection.** Without a queue, selecting by diff would let a documentation commit show green above a commit that broke the core, and a cancelled run would leave that break unchecked. Running everything costs about a minute on documentation pushes, on free runners.
- **Pull requests cancel superseded runs, and every push to `main` gets a run of its own.** On `main` a run per push says which commit broke it, and on free runners the runs go side by side. That needs a concurrency group per commit on `main`, set in `82d7f2c`. A shared group with `cancel-in-progress` off, as `813ea75` had, still cancels a pending run when a newer one arrives, and makes the rest wait in line behind the run in progress.
- **The pre-push hook is the gate before `main`.** It runs format, lint, the conventions check and types on what the push changes, in under 9 seconds. It is a hook and not a line in `CLAUDE.md`, because agents skip steps.
- **Time budgets, enforced.** A push gets its result in under 2 minutes at the median and under 3 at the 95th percentile. Each job fails when a warm run exceeds its budget. A cold run is capped by the job's timeout. A cold run happens when a new Rust toolchain invalidates the cache, about once every six weeks, and it is reported as cold.
- **Releases are free and unsigned until there is an Apple Developer account. This one waits on the owner.** A tag builds the disk image, signed ad hoc, and publishes it as a GitHub prerelease with checksums and a provenance attestation. It would amend the Distribution section of ADR 0012, which says distribution is not settled, so ADR 0012 stays as it is until the owner confirms. There is no updater, because ADR 0013 forbids update checks.

## The brief's defaults, applied here

The brief this plan answers names default tools and techniques, and asks for a measured reason to keep anything else.

| Default | Here | Reason |
| --- | --- | --- |
| Move to Blacksmith for waiting under 10 s | Not adopted | Measured waiting is 1–3 s. The owner chose free runners. |
| Cancel a branch's run when a new commit lands | On for pull requests | Each push to `main` keeps its own run, since `82d7f2c`. |
| Cache hits above 95 %, seeded from `main` | Ticket 01 | The biggest bucket. |
| Cache `node_modules` | Not adopted | `bun install` takes 2–3 s from bun's own cache. |
| Blobless clone | Not adopted | Nothing reads history, and the shallow checkout takes 1–2 s. |
| oxlint and oxfmt | Ticket 04 compares them with Biome side by side | Biome lints and formats the whole UI in 40 ms, CSS and JSON included. Switch only on what the comparison shows. |
| `bun check` for types | Ticket 04 | 70 ms against `tsc` 7's 90 ms, same errors, annotations on GitHub. No reason to keep `tsc`. |
| tsrs locally | Not adopted | Announced in October 2026 and experimental. |
| Install and scripts with bun | Already on | |
| Vitest `isolate: false`, sharding, one database per shard | Not adopted | One test file, 1 s. The core's tests take 2 s and use a temporary directory each. Revisit when a test job passes 30 s. |
| Tests in random order | Ticket 05, for the UI | Rust's own shuffle is nightly-only. |
| Lint, types, build and tests in parallel jobs | Only the macOS job splits (ticket 02) | The Linux job's checks take 8 s together, and each extra job costs about 25 s of setup. The macOS job is the critical path. |
| Run only what the diff can reach | Not adopted | No pull requests and no queue. See "Decisions". |
| Pre-push hook under 9 s | Ticket 03 | |
| Fail any job over its budget | Ticket 07 | |

## Tickets

| Ticket | What | Blocked by |
| --- | --- | --- |
| [01](issues/01.md) | The Rust cache hits, and ADR 0021 records how CI works | None |
| [02](issues/02.md) | The macOS job splits in two | 01 |
| [03](issues/03.md) | A pre-push hook guards `main` | None |
| [04](issues/04.md) | `bun check` for types, and Biome against oxlint and oxfmt | None |
| [05](issues/05.md) | The UI's tests run in random order | None |
| [06](issues/06.md) | The workflows are locked down and dependencies watched | None |
| [07](issues/07.md) | Every job has a time budget that fails it | 02 |
| [08](issues/08.md) | A tag publishes the disk image | 06 |

Each ticket is one commit to `main`. Its comments record the before and after from the CI run of that commit, by run ID.

## Before and after

| | Before, measured | After, projected |
| --- | --- | --- |
| Push to result, typical | 6 min 21 s cold, 2 min 29 s partial, 1 min 32 s – 1 min 47 s warm | about 1 min 20 s, warm on every image |
| Push to result, `Cargo.lock` changed | 2 min 29 s | about 2 min 30 s |
| Push to result, new Rust toolchain | 6 min 21 s | about 4 min 30 s, the two macOS jobs in parallel |
| Runner minutes per push | 7.6 cold, 2.8 warm | about 2.6 warm |
| Cost per month | $0 | $0 |

The projections come from the step timings above and are confirmed or corrected by each ticket's own run.

## Out of Scope

- Paid runners, a GitHub organization and a merge queue.
- Selecting checks by diff.
- Signing with a Developer ID and notarization. They need the Apple Developer Program, $99 a year. The release workflow leaves a place for them.
- An updater inside the app (ADR 0013).
- Intel and universal builds, Homebrew, and Linux or Windows builds (ADR 0012).
- Restoring file times from git to skip rebuilding workspace crates. A commit authored before the previous cached build would look up to date, and CI would test stale code.

## Further Notes

- The repository is public, so GitHub-hosted standard runners, the Actions cache (10 GB) and artifact attestations are free. If it ever goes private, macOS minutes cost about ten times Linux minutes and this plan's costs change.
- The `.github` folder and the workflow files already pass the naming rule. A git hook named `pre-push` does not, and git dictates the name, so ticket 03 adds it to the conventions check's dictated names.
- The no-comments rule of ADR 0015 applies to workflow files too. Actions are pinned by commit SHA without a version comment. Dependabot reads the SHA.
