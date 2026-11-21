# Release: the first release, measured, fast to start, and installable

**Status:** needs-info

## Problem Statement

ADR 0009 fixes the budgets every change respects, and the list spike measured where they stand (`documentation/research/list.md`):

- **Within budget:** idle memory, the installer, the JavaScript and interactions.
- **Cold start:** misses its 300 ms by 10 to 40 ms, inside Tauri's start and the page's script.
- **The list:** drew at 60 fps, because WKWebView caps pages near 60. ADR 0023 lifts the cap through a private WebKit switch, and it now draws at 120.

The budgets for memory and frames are measured by hand, so they can erode quietly, which ADR 0009 asks to prevent. ADR 0012 leaves distribution unsettled: macOS turns most people away from an app that is not signed and notarized, which needs an Apple Developer account at 99 dollars a year.

## Solution

A command measures every budget of ADR 0009 on the machine it runs on, against a generated board of 100,000 messages, and fails when one breaks. Cold start is brought under 300 ms. The list draws at 120 Hz (ADR 0023). The owner decides whether to pay for signing, and once that is settled, a tag publishes the disk image.

## User Stories

1. As a contributor, I want one command that measures every budget in ADR 0009 and fails when one breaks, so that the app never gets slower or heavier by accident.
2. As an operator, I want the app usable within 300 ms of launch, so that it feels instant (ADR 0009).
3. As an operator, I want the list to move at my display's rate, so that scrolling feels as smooth as the rest of macOS (ADR 0009).
4. As an operator, I want to install the app without macOS warning me off it, so that I trust it.
5. As a maintainer, I want a tag to publish the disk image, so that releasing is one step.

## Implementation Decisions

- **The measuring command:**
  - It generates a board of 100,000 messages with `vb serve` in a temporary folder and builds the app in a measuring mode.
  - It measures as the spike did: frames while the list scrolls itself, jumps, cold start from `main`, and memory with `footprint` across the app and its WebKit processes, over several runs.
  - It compares each result with ADR 0009.
  - The measuring mode is compiled only into that build, not into the release.
  - It runs by hand and before a release. CI's runners cannot measure frames or WebKit's memory reliably.
- **Cold start:**
  - Find what Tauri does in its 170 ms before `setup`, and trim it.
  - Trim the page's script, about 110 ms to the first frame: load less before the first paint, and defer what the first screen does not need.
  - If 300 ms turns out unreachable on Tauri, the number is revised in ADR 0009 with the measurements, as that ADR asks.
- **120 Hz** was chosen by the owner on 2026-11-21 and is built: the shell lifts WebKit's 60 fps cap (ADR 0023).
- **Signing** waits on the owner's decision. The pipeline's ticket 08, publishing the disk image on a tag, is `wontfix` until then.

## Testing Decisions

- The measuring command is tested by running it, and its results are kept in `documentation/research/list.md` as the budgets' history.
- Cold start is measured with that command, before and after.

## Out of Scope

- An update check, which ADR 0013 says needs its own decision and has to be visible to the person.
- Windows, Linux, the browser binary and Docker (ADR 0012).

## Open Questions

- **Pay for an Apple Developer account,** 99 dollars a year, to sign and notarize the app? Without it, macOS warns everyone who opens it.
