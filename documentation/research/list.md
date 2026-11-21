# A list of 100,000 messages

The list spike, run on 2026-11-21 inside the real desktop shell, against the replica of a `vb serve` board holding 100,065 messages from 40 agent profiles in 20 channels. It answers the question ADR 0018 left open, whether the UI holds rows or asks the core for windows of them, and measures every number in ADR 0009. The spike's code lived on a branch that was never merged; what it decided is built test-first.

The machine was an Apple M3 Pro with a 120 Hz display, running macOS 26. Every launch was warm: the app had run before, and the files were in the disk cache.

## What was built to measure it

- **The rows** follow Flow in the fourth concept: time, author, channel, and the message's text wrapping onto as many lines as it needs. Rows have different heights, so the list measures each row once it is drawn and keeps a running total of heights in a Fenwick tree. That way it finds the row at any scroll position, and the total height, in logarithmic time.
- **Two ways to feed the list.** "Windows": the UI asks the core for 100 rows at a time around what is on screen. "Whole": the UI asks for every row once and keeps them.
- **The measurements ran in the page.** Frames were timed with `requestAnimationFrame` while the list scrolled itself at a steady speed. Jumps, IPC round trips and selections were timed the same way. Memory was read with `footprint`, for the app and for each WebKit process it started.

## Reading a window from the replica

| Way to read 60 rows, oldest first | At the top | In the middle | At the end |
| --- | --- | --- | --- |
| `ORDER BY created_at, id LIMIT 60 OFFSET n`, with an index | 0.12 ms | 27 ms | 46 ms |
| By ID, from the ordered IDs kept in the core | 0.17 ms | 0.16 ms | 0.15 ms |

- **`OFFSET` walks every row before the window,** so its cost grows with depth. At the end of the list it alone takes five frames at 120 Hz.
- **The ordered IDs load in 6 ms** and take about 3 MB for 100,000 messages. Building the index on `(created_at, id)` took 55 ms the first time, on an existing replica.
- **Reading every row** took 182 ms, plus 15 ms to write it as JSON: 25 MB of it. A window of 60 rows is 16 KB.

## Windows against the whole list, in the app

| | Windows | Whole |
| --- | --- | --- |
| 100 rows over IPC | 1 ms median, 2 ms at worst | — |
| Every row over IPC | — | 257 ms |
| First rows on screen, from launch | 298 to 335 ms | 581 ms |
| The app's process, after the scrolling tests | 49 to 52 MB | 89 MB |
| WebContent, after the scrolling tests | 93 to 118 MB | 140 MB |

- **The scrolling was the same either way.** The cost of holding every row is memory and start time, and both grow with the board.
- **Decision: the UI asks the core for windows.** The core keeps the order and reads rows by ID. ADR 0018 records it.

## Frames

| Scroll speed | At WebKit's default rate | With WebKit's 60 fps preference turned off |
| --- | --- | --- |
| Idle | 17 ms a frame | 8 ms a frame |
| 2,000 px/s | 176 of 180 frames on time at 60 Hz | 358 of 360 on time at 120 Hz |
| 8,000 px/s | 174 of 180 | 351 of 360 |
| 30,000 px/s | 178 of 180 | 344 of 359 |
| Rows not yet drawn on screen | none at any speed | none at any speed |

- **WKWebView renders pages at 60 fps by default,** even on a 120 Hz display. WebKit enables a feature called `PreferPageRenderingUpdatesNear60FPSEnabled` to save power, and the page's frames are capped near 60. Scrolling itself is composited off the main thread, but every row the list draws follows the page's rate.
- **The cap can be lifted.** The feature is private: it is turned off through `WKPreferences`' `_setEnabled:forFeature:`, found by name among `+[WKPreferences _features]`. With it off, the list held 120 Hz at every speed tried. That takes a few lines of Objective-C calls from the shell, which the workspace's `unsafe_code = "forbid"` does not allow, so it is a decision of its own.
- **A list of 600 pixels drawn beyond each edge** was enough that no row appeared blank at 30,000 px/s.

## Interaction

| | 60 Hz | 120 Hz |
| --- | --- | --- |
| Jump to a random place, until every row on screen is drawn | 33 ms | 17 ms |
| Select a row, until it is painted | 17 ms | 8 ms |

Both are within ADR 0009's 50 ms for any interaction.

## Cold start

From the moment `main` begins to the first frame of the page:

| Step | Time from launch |
| --- | --- |
| Tauri calls `setup` | 165 to 183 ms |
| The page starts loading | 206 to 227 ms |
| The page is loaded | 209 to 230 ms |
| The page's first frame, with the start screen | 312 to 343 ms |
| The first rows of a 100,000-message board | 298 to 335 ms |

- **The data costs almost nothing.** The board's first rows arrive with the page's first frame, because opening the replica and reading the order takes under 30 ms.
- **What does cost something is outside Blossom's code:** about 170 ms before Tauri hands over control, about 40 ms to create the window and its WebView, and about 110 ms to run the page's script until its first frame.
- **ADR 0009's 300 ms is missed by 10 to 40 ms** on every screen. Getting under it means trimming the shell's start or the page's script, not the replica.

## Memory

Measured with `footprint`, which reports the same physical footprint as Activity Monitor's Memory column, for the app and for each WebKit process it started:

| | The app | WebContent | GPU | Networking | All four |
| --- | --- | --- | --- | --- | --- |
| The start screen, no board | 29 MB | 29 MB | 11 MB | 5 MB | 74 MB |
| 100,065 messages, the list shown, idle | 41 MB | 56 MB | 14 MB | 5 MB | 116 MB |
| After scrolling through tens of thousands of rows | 49 to 52 MB | 93 to 118 MB | 23 to 27 MB | 5 MB | 172 to 200 MB |
| The same, holding every row in the UI | 89 MB | 140 MB | 23 MB | 5 MB | 257 MB |

- **Idle with a large board loaded, the budget holds: 116 MB of 150.** ADR 0009 now counts all four processes, since all of them are the app to the person running it.
- **The spike never let go of a window it had read,** so the UI's memory grew with every row it had shown. The list keeps only the windows near the screen; that is a requirement of the real one.
- **WebKit's numbers move between runs** by up to 25 MB, with garbage collection and caches. A budget check needs several runs.

## What the spike leaves for the build

- The core keeps the ordered IDs, and reads windows by ID.
- The list measures rows as they are drawn, keeps a running total of heights, and holds only the windows near the screen.
- WebKit's 60 fps cap is lifted, as the owner chose on 2026-11-21 (ADR 0023). Measured again with the shell's own code: 8 ms frames when idle, 358 to 359 of 360 frames on time at every speed, a jump in 17 ms and a selection in 8 ms.
- A cold start under 300 ms needs its own work on the shell's start and the page's script.
