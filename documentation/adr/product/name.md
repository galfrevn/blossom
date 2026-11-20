---
adr: 0014
status: accepted
date: 2026-11-20
---

# The project is called Blossom

The project is named Blossom. Where the name needs context, such as the repository description, a website or a package description, it is written "Blossom for Vercel Board". "Vercel" never goes inside the name itself.

The owner chose it for how it sounds: "Yes, I like Blossom." It does not describe what the product does, and its soft tone is unlike the visual identity in ADR 0006. Both points were raised before the choice.

## Considered options

- **Atalaya.** Proposed by Claude as a working name and rejected: "No, horrible."
- **Veneer.** A thin layer laid over a board. Claude recommended it. Not chosen.
- **Masthead.** A head for a headless board. Not chosen.
- **Also shortlisted:** Inlay, Lacquer, Grain, Quarterdeck, Muster, Bosun, Bevel, Loupe, Dither, Overboard.

## Consequences

- `blossom` is already taken on npm and crates.io, so packages will need a scope or a suffix.
- Other projects use the name. The best known are a self-hosted Markdown editor (`blossom-editor/blossom`, about 3,800 stars on 2026-11-20) and a media storage protocol for Nostr (`hzrd149/blossom`). Writing "for Vercel Board" keeps this project distinct in search.
- Publishing the repository no longer waits for a name.
- The two artifacts from the brainstorm were renamed to Blossom on 2026-11-20.
