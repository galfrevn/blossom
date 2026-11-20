# Domain docs

How skills read and write this repository's domain documentation.

## Where things live

- **`GLOSSARY.md`** at the repository root: the project's language. There is one context, so there is no `GLOSSARY-MAP.md`.
- **`documentation/README.md`**: the index of every ADR, with its number, status and file.
- **`documentation/adr/<area>/<word>.md`**: one decision per file. The areas so far are `product`, `architecture` and `design`.

## Before exploring

Read `GLOSSARY.md`, then the ADRs in the index that touch the area you are about to work in.

## Use the glossary's vocabulary

When your output names a domain concept (a ticket title, a refactor proposal, a hypothesis, a test name, an identifier), use the term as `GLOSSARY.md` defines it. Stay off the synonyms it lists under _Avoid_.

A concept that is missing from the glossary is a signal: either you are inventing language the project does not use, or there is a real gap to raise with the `domain-modeling` skill.

## Recording a decision

An ADR here differs from the skill's default in three ways:

1. Its file name is one word, inside the folder of its area. There is no number and no slug in the name.
2. Its number, status and date go in frontmatter: `adr`, `status`, `date`.
3. It gets a row in the index in `documentation/README.md`. The next number is the highest in that index plus one.

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly instead of silently overriding it:

> _Contradicts ADR 0007 (React for the web UI), but worth reopening because…_
