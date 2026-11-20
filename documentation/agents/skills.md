# Skills

Which skill to reach for, and when. They are Matt Pocock's skills, installed under `.claude/skills` and adjusted to this repository's layout. Skills marked "you" only run when a person types them; the rest can also be picked up by the agent.

## The main flow

| Step | Skill | Who starts it | What comes out |
| --- | --- | --- | --- |
| Settle a design | `/grill-with-docs` | you | Decisions in ADRs, terms in the glossary |
| Answer a design question cheaply | `prototype` | either | A throwaway prototype |
| Gather facts | `research` | either | A findings file under `documentation/research` |
| Write the spec | `/to-spec` | you | `.scratch/<feature>/spec.md` |
| Break it into tickets | `/to-tickets` | you | `.scratch/<feature>/issues/<NN>.md` |
| Build a ticket | `/implement` or `/implement-spec`, with `tdd` | you | Code, test-first |
| Shape a module | `codebase-design` | either | A deeper interface |
| Review the change | `/code-review` | you | Claude Code's own review skill |
| Describe the change | `pr` | either | A pull request body |

`/grill-with-docs` runs two skills together: `grilling` asks the questions and `domain-modeling` writes the answers down. `/implement`, `/implement-spec` and `tdd` finish by calling `code-review`, which here resolves to the one Claude Code ships.

## Around the flow

- `diagnosing-bugs`: a bug or a performance regression that resists a quick fix.
- `/handoff`: compact a conversation so a fresh agent can continue.
- `/wayfinder`: work too big for one session, planned as a map of decision tickets.
- `/improve-codebase-architecture`: scan the code for modules worth deepening. Useful once there is code.
- `writing-for-agents`: when editing `CLAUDE.md` or a skill.
- `/setup-matt-pocock-skills`: already done by hand. Kept for its GitHub tracker template, for when tickets move to GitHub Issues.

## Removed or never installed

Eight of Matt Pocock's skills were removed because nothing in this project's flow uses them: `ask-matt`, `grill-me`, `retro`, `teach`, `to-questionnaire`, `triage`, `wait-what` and `wizard`. They can be copied back from `mattpocock/skills` if that changes; `triage` is the likely one, once the repository receives issues.

Two were never installed:

- Matt Pocock's `code-review`: it collides with the review skill Claude Code ships.
- Vercel's own `board` skill: it teaches an agent to post on a team's board as it works. In this repository "board" is the subject of almost every conversation, so it would fire constantly and write to a real board.
