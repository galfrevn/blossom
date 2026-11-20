---
adr: 0006
status: accepted
date: 2026-11-20
---

# Dark glass, dense, with pixel and dither accents

The visual direction was settled over three concept iterations, against a screenshot of the T3 Code desktop app as the reference. The interface is dark translucent glass with blur, set in small dense type (13px base), with hairline dividers between everything, a dithered pixel header, and Geist, Geist Mono and Geist Pixel as the typefaces. It should read as a developer tool in the vein of Cursor, not as a consumer app.

The direction is accepted with refinements pending. The verdict on the third iteration was: "There are things to improve but it is much better."

## Considered options

- **A sober monochrome instrument** with light and dark themes (first iteration). Superseded once a reference was provided.
- **Apple-like**: large system type, wide radii and a lot of air (second iteration). Rejected outright. The correction asked for smaller text, more glass, more of a developer feel, more dividers, and pixel and dither styling.

## Consequences

- Geist Pixel only reads as a pixel face from about 18px up. At 11 to 13px it looks almost like a regular sans, so it is used for titles, large figures and the wordmark.
- Geist Pixel is not on Google Fonts. It has to be bundled with the app (SIL Open Font License, from the `geist` package on npm).
- The product has both a light and a dark theme. The third iteration only has the dark one, so the light theme still has to be designed: glass and dither were drawn for dark and need their own treatment in light.
- The improvements still wanted in the third iteration are polish. They do not reopen this direction, and they have not been listed yet.
