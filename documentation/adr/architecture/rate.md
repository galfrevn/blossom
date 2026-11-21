---
adr: 0023
status: accepted
date: 2026-11-21
---

# The page draws at the display's rate, through WebKit's own switch

ADR 0009 asks for a list of 100,000 messages that scrolls at 120 fps. WKWebView draws pages near 60 fps even on a 120 Hz display, because WebKit enables a feature called `PreferPageRenderingUpdatesNear60FPSEnabled` to save power. The list spike measured it: 60 fps as WebKit ships (`documentation/research/list.md`). The owner chose 120 on 2026-11-21.

When the window is set up, the shell turns that feature off for its WebView. It finds the feature by name among `+[WKPreferences _features]` and switches it with `-[WKPreferences _setEnabled:forFeature:]`. Both are private WebKit calls, made through `objc2`, which Tauri already depends on, so the app gains no weight.

## Consequences

- **`unsafe` is allowed in one function, and nowhere else.** The workspace denies `unsafe_code` instead of forbidding it, so that `lift_the_sixty_fps_cap` in `desktop/src/rate.rs` can allow it. The core and the conventions crate keep `#![forbid(unsafe_code)]` of their own.
- **If Apple renames or removes the feature,** the app finds nothing to switch, does nothing, and draws at 60 fps again. Each call it makes is checked with `respondsToSelector:` first, and a null answer stops it.
- **Drawing at 120 Hz costs more power while something moves.** An idle page draws no frames either way.
- **Measured with the shell's own code** against 100,065 messages:
  - frames took 8 ms when idle;
  - 358 to 359 of 360 frames were on time at 2,000, 8,000 and 30,000 px/s, with no blank rows;
  - a jump anywhere took 17 ms, and selecting a row 8 ms.
- **The App Store would refuse private calls.** The app is not distributed there (ADR 0012).

## Considered options

- **60 fps, as WebKit ships, with ADR 0009's list budget revised to 60.** The scrolling was smooth at 60 too, with 97 to 99% of frames on time. Rejected by the owner: speed is the product's promise.
- **Waiting for a public API.** None exists, in WebKit or in Tauri.
