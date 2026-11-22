---
adr: 0009
status: accepted
date: 2026-11-20
---

# Weight and speed budgets that every change has to respect

The software has to weigh almost nothing and be very fast and light. To make that defensible when someone wants to add a dependency or a feature, it is fixed as numbers.

| Budget | Target |
| --- | --- |
| Desktop installer | under 15 MB |
| Idle memory, with a large board loaded | under 150 MB |
| The UI's JavaScript, compressed | under 250 KB |
| Cold start to a usable screen | under 300 ms |
| Any interaction | under 50 ms |
| A list of 100,000 messages | scrolls at 120 fps |

These are targets. The spikes measured them in November 2026; the numbers are in `documentation/research/board.md` and `documentation/research/list.md`.

## How each is measured

- **Memory** is the physical footprint, as `footprint` and Activity Monitor report it, of the app and of every WebKit process it starts: the content, the GPU and the networking processes. To the person running the app they are all the app. "Idle with a large board loaded" means a board of 100,000 messages, its replica live, the list on screen, and nothing scrolled.
- **Cold start** runs from the moment `main` begins to the first frame of a screen a person can use, measured in the page.
- **Frames** are timed in the page while the list scrolls by itself, so they count what the list draws, not only what the system composites.

## Where they stand

| Budget | Measured | |
| --- | --- | --- |
| Desktop installer | 4.6 MB | holds |
| Idle memory, with a large board loaded | 116 MB | holds |
| The UI's JavaScript, compressed | 98 KB, 17 of them markdown (Thread 02) | holds |
| Cold start to a usable screen | 310 to 340 ms | missed: Tauri's own start takes about 170 ms, and the page's script about 110 |
| Any interaction | 17 to 33 ms | holds |
| A list of 100,000 messages | 120 fps, with WebKit's 60 fps cap lifted (ADR 0023), and no blank rows | holds |

## Consequences

- A change that breaks a budget needs a reason written down, or it does not go in.
- The budgets should be checked automatically once there is something to build, so they do not erode quietly.
- If a spike shows a number is unrealistic, the number is revised here rather than ignored.
