---
adr: 0021
status: accepted
date: 2026-11-20
---

# CI checks everything on every push, on free runners, in under two minutes

The owner decided the first two points below on 2026-11-20. The rest were recommended in the pipeline plan (`.scratch/pipeline/spec.md`) and are recorded as accepted for review.

One person pushes straight to `main`, so CI is the only thing between a commit and `main`. CI runs the whole suite on every push, on GitHub's free runners. A pre-push hook stops the cheap mistakes before they reach it, and each job has a time budget that fails it, so CI cannot quietly get slower.

- **Free standard runners.** `ubuntu-latest` and `macos-latest` cost nothing on a public repository. Jobs waited 1–3 seconds for a runner, so a faster or paid runner would not shorten that wait.
- **Pushes go straight to `main`.** There is no pull request gate and no merge queue. Pull requests from Dependabot or outside contributors run the same suite.
- **The whole suite on every push, with nothing skipped based on the diff.** Without a queue, choosing checks by what changed would let a documentation commit show green on top of a commit that broke the core, and a cancelled run would leave that break unchecked. A documentation push costs about a minute.
- **Every push to `main` gets a run of its own.** Each run then says which commit broke `main`, and on free runners the runs go side by side. Pull requests cancel their superseded runs.
- **A pre-push hook is the gate before `main`.** It checks format, lint, the conventions and types on what the push changes, in under 9 seconds. It is a hook rather than a line in `CLAUDE.md` because agents skip steps, and `--no-verify` is never used.
- **Only pushes to `main` write the Rust cache.** Pull requests read it and never write their own.
- **The Rust cache depends only on the pinned toolchain and the lockfile.** Before the cache is restored, CI removes every toolchain except the pinned one. `Swatinem/rust-cache` hashes every toolchain installed on the runner into its key. Runner images ship different extras (image `20261102` of `macos-26-arm64` carried Rust 1.99.1, and `20261117` did not), so the same commit got two keys and missed the cache during each image rollout.
- **Time budgets.** A push gets its result in under 2 minutes at the median and under 3 at the 95th percentile. A job fails when it runs over its budget on a full cache match. A run that starts with a cold cache is reported as cold and is capped by the job's timeout. It happens when a new toolchain invalidates the cache, about once every six weeks. No retries, no wider timeouts, no skipped tests.

| Job | Budget, warm | Timeout |
| --- | --- | --- |
| `linux`: conventions, core and UI | 70 s | 3 min |
| `desktop`: Clippy on the shell | 60 s | 3 min |
| `installer`: the disk image | 150 s | 8 min |

A budget is measured from the job's first step to its last check, and it is about 1.3 times the slowest warm run measured. A timeout is about 1.5 times the slowest cold run, rounded up to whole minutes. Neither is raised to make a run pass. The measurements are in `.scratch/pipeline/issues/07.md`.

## Considered options

- **Turning off `rust-cache`'s environment hash** (`add-rust-environment-hash-key: false`) and putting the toolchain in the key by hand. Rejected: the same switch drops the hash of `Cargo.lock` and every `Cargo.toml`. After a dependency change, the old cache would restore as a full match, `rust-cache` would skip saving it, and every later run would rebuild the changed crates.
- **Blacksmith or other paid runners.** Rejected: the owner chose free runners, and waiting for a runner is already 1–3 seconds.
