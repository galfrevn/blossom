---
adr: 0006
status: accepted
date: 2026-11-20
---

# Dark, dense, with pixel and dither accents

The visual direction was settled over three concept iterations, against a screenshot of the T3 Code desktop app as the reference. The interface is solid, not translucent, set in small dense type (13px base), with hairline dividers between everything, a dithered pixel header, and Geist, Geist Mono and Geist Pixel as the typefaces. It should read as a developer tool in the vein of Cursor, not as a consumer app.

The verdict on the third iteration was: "There are things to improve but it is much better." The fourth concept, version 5 of the interactive concept, was approved as it stands on 2026-11-21: "I like it." It is in English, has light and dark themes and no glass, and shows the four lenses of the first release. The lenses are built on it.

## Glass, postponed

The third iteration was translucent glass with blur, and this decision first said so. On 2026-11-20, before the app was scaffolded, glass was taken out: "Forget it, no glass. Let's leave it as an addition for later, once we have a working product." It comes back, if it does, after the first release works. There are two kinds to choose from then: the window's own vibrancy on macOS, which shows the desktop through the window and needs it to be transparent, and blur inside the page for floating panels such as the command palette.

## Considered options

- **A sober monochrome instrument** with light and dark themes (first iteration). Superseded once a reference was provided.
- **Apple-like**: large system type, wide radii and a lot of air (second iteration). Rejected outright. The correction asked for smaller text, more glass, more of a developer feel, more dividers, and pixel and dither styling.

## Consequences

- Geist Pixel only reads as a pixel face from about 18px up. At 11 to 13px it looks almost like a regular sans, so it is used for titles, large figures and the wordmark.
- Geist Pixel is not on Google Fonts. It has to be bundled with the app (SIL Open Font License, from the `geist` package on npm).
- The product has both a light and a dark theme, and the fourth concept draws both, dither included.
- The improvements once wanted in the third iteration were never listed. The fourth concept was approved without them, so none are pending.
