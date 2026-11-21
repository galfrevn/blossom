# Blossom

A visual layer for Vercel Board: a macOS desktop app with a Rust core and a React UI. Its decisions are indexed in `documentation/README.md` and its vocabulary is in `GLOSSARY.md`.

Before changing an area, read the ADRs that cover it. When a change would contradict one, say so instead of overriding it.

## How code is written here

When quality and speed of delivery conflict, quality wins.

**English everywhere.** Code, identifiers, file names, commit messages, documents and user-facing strings.

**Commits are conventional and lowercase.** Every commit message follows Conventional Commits and is written entirely in lowercase, body included: `feat(replica): merge events by version`. The message ends with the body: it carries no co-author or other attribution trailer.

**Names carry the meaning.** The source has zero comments: no line comments, no doc comments, no TODOs, no commented-out code. When a line needs explaining, rename or extract until it reads on its own. A reason that has to outlive the code goes in an ADR.

**Long and obvious beats short and clever.** Identifiers are full words that say what the thing is: `unreadMessageCount`, `leaseExpiresAt`, `index`. Prefer named intermediate values and early returns to chained one-liners and nested conditionals. Domain concepts use the exact terms in `GLOSSARY.md`.

**One word per file, folders for the rest.** Every file and folder name is a single lowercase English word. When a name wants a second word, the first becomes a folder: `attention/list.tsx`, `replica/merge.rs`. A test file sits in a `tests` folder beside the code it tests: `attention/tests/list.tsx` tests `attention/list.tsx`. The only exceptions are names a tool dictates (`Cargo.toml`, `package.json`, `README.md`); where the tool lets the name be configured, configure it.

**No slop.** Every line earns its place:

- Build what the task needs. An abstraction waits for its second caller.
- A function either works or does not exist: no stubs, no placeholders, no dead code.
- Delete what a change replaces.
- Types are explicit at module boundaries. `any`, `unwrap` and `expect` stay inside tests.
- Each new dependency is weighed against the budgets in `documentation/adr/architecture/budgets.md`.

**Built for a fleet.** Every list, query and render path assumes hundreds of agents and 100,000 messages.

**Tests first.** Behaviour is specified by a failing test before the code that satisfies it.

**Done means `bun run check` passes.** It runs everything CI runs, including the check of these naming and comment rules.

**Every push goes through the pre-push hook.** When it refuses a push, fix what it reports and push again; `--no-verify` is never used.

## Agent skills

### Issue tracker

Specs and tickets are markdown files under `.scratch/`. See `documentation/agents/tracker.md`.

### Triage labels

The five default roles, unchanged. See `documentation/agents/labels.md`.

### Domain docs

Single context: `GLOSSARY.md` at the root, ADRs under `documentation/adr/`. See `documentation/agents/domain.md`.
