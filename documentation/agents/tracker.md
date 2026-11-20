# Issue tracker: local markdown

Specs and tickets for this repository live as markdown files under `.scratch/`. The repository is on GitHub, but its tickets stay in local files for now. Moving them to GitHub Issues means replacing this file.

## Conventions

- One feature per directory, named with one word: `.scratch/<feature>/`
- The spec is `.scratch/<feature>/spec.md`
- Tickets are one file each at `.scratch/<feature>/issues/<NN>.md`, numbered from `01`. The title is the file's first heading, not part of its name. There is never a single combined tickets file.
- Triage state is a `Status:` line near the top of each ticket. The role strings are in `labels.md`.
- Comments and conversation history append to the bottom of the file under a `## Comments` heading.

## When a skill says "publish to the issue tracker"

Create a new file under `.scratch/<feature>/`, creating the directory if needed.

## When a skill says "fetch the relevant ticket"

Read the file at the referenced path. The path or the ticket number is normally passed directly.

## Wayfinding operations

Used by `/wayfinder`. The **map** is a file with one **child** file per ticket.

- **Map**: `.scratch/<effort>/map.md` (the Notes / Decisions-so-far / Fog body).
- **Child ticket**: `.scratch/<effort>/issues/<NN>.md`, numbered from `01`, with the question in the body. A `Type:` line records the ticket type (`research`/`prototype`/`grilling`/`task`); a `Status:` line records `claimed`/`resolved`.
- **Blocking**: a `Blocked by: NN, NN` line near the top. A ticket is unblocked when every file it lists is `resolved`.
- **Frontier**: scan `.scratch/<effort>/issues/` for files that are open, unblocked and unclaimed; first by number wins.
- **Claim**: set `Status: claimed` and save before any work.
- **Resolve**: append the answer under an `## Answer` heading, set `Status: resolved`, then append a context pointer (gist plus link) to the map's Decisions-so-far in `map.md`.
