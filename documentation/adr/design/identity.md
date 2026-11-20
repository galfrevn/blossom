---
adr: 0016
status: accepted
date: 2026-11-20
---

# Soft paper, warm ink, and a cherry blossom accent

The identity rests on the direction the owner gave on 2026-11-20:

- **Neutrals.** Light and dark themes, neither in pure white nor pure black. Light is a very soft cream; dark is a warm ink whose text is that same cream.
- **Accent.** A cherry blossom pink that stays out of the way, because this is a tool for developers and agents. It has to be easy to change.
- **Texture.** Dither, pixelated gradients, ASCII and pixel art.
- **Type.** Geist Pixel for display, Geist for the interface, Geist Mono for code. The wordmark is set in Geist Pixel Square, chosen on 2026-11-20 over Grid, Circle, Triangle and Line.
- **Mark.** The dithered cherry blossom, B1 in the first draft, chosen on 2026-11-20.

## How it is built

- Colors are defined in OKLCH. Every pink comes from two values, `--accent-hue` and `--accent-chroma`, with lightness fixed per theme so contrast holds whatever the hue.
- The accent is limited to the mark, unread markers, the selected row, counts and the focus ring. Primary actions are ink, not pink. States such as blocked or failed will get their own semantic colors and never use the accent.
- Gradients are ordered dither on a Bayer 8×8 grid. The coarse Bayer 4×4 strip was shown and rejected. Textures sit at edges and in empty space, never behind text that has to be read.
- ASCII is kept as a density ramp for the README and the terminal. The ASCII empty state was shown and rejected.
- Geist Pixel is used from 18px up, as ADR 0006 already says.

## The mark

A five-petal cherry blossom with notched petals, filled with ordered dither. It is deep at the core and fades toward the tips. Its edges are dithered instead of anti-aliased. All three tones are accent tokens (`--petal-deep`, `--petal`, `--petal-soft`), so the mark changes color with the accent. When only one color is available, the ink replaces all three.

The mark has three drawings, one for each range of sizes:

- **16 grid**, for anything below 32px. It is drawn by hand.
- **32 grid**, from 32 to 79px.
- **40 grid**, from 80px up.

Each drawing is shown only at whole multiples of its grid, with crisp edges. A drawing is never scaled to another size. The wordmark sits beside the mark, separated by half the mark's width.

The tokens live in `ui/theme/tokens.css`. The app imports that file as it is. The default accent is hue 354 and chroma 0.105.

The reference sheet is published at https://claude.ai/artifact/PBVZyL5LWNNxLAZTMB2Aty. Only its owner can open it. Its first two versions keep the drafts.

## Considered options

- **Eight other lone blossoms** from the first draft: a hand-drawn 16 grid, a 32 grid, an outline, ASCII, halftone, a vector shape, a single petal and scattered pixels. B1 was preferred.
- **Eight trees made the same way as B1** in the second draft: bonsai with cloud pads, a dome, a cascade and a windswept tree; B1 as the crown of a tree; a branch; a tree in front of a moon; and an open-air cherry tree. They were dropped with "Forget it, let's go with the blossom I chose before."

## Consequences

- This refines ADR 0006 and does not replace it. The interface stays dense, with hairline dividers; glass is postponed until the first release works, as ADR 0006 now says. The light theme is now defined as cream paper.
- Shrinking the large drawing loses its detail below 32px, so the 16 drawing has to be kept up to date by hand if the mark ever changes.
- The token names in `ui/theme/tokens.css` are a contract with the UI: renaming one breaks the scaffold's mapping onto Tailwind.
