# Scaffold: the skeleton the spikes and the app are built on

**Status:** ready-for-agent

## Problem Statement

There is no code yet. Before any spike can measure anything, and before any lens can be built, the project needs a working path from a board on the operator's Mac to a window that shows it: a Rust core, a Tauri shell, a React UI, the contract between them, and the checks that keep the rules of `CLAUDE.md` and the budgets of ADR 0009 from eroding. Without that path, every spike would first have to build its own throwaway version of it, and the conventions would depend on people remembering them.

The operator's first contact with the app also has to work from the first build. ADR 0012 promises a first run with nothing to configure: the app looks for a local board and opens it without asking, and when there is none, it shows how to start one.

## Solution

A walking skeleton: the thinnest slice that crosses every layer and does something real. When the operator opens Blossom, it finds the local boards that `vb serve` is running on their Mac, opens the one started most recently, and shows which board is open and who they are on it. When no board is running, it shows how to start one. When a board starts or stops while the app is open, the app follows within about a second, without a restart.

Around that slice, the repository gets its toolchain: one command to develop, one to run every check, one to build; conventions checked by machine; CI on every push, with the two size budgets measured from the first build.

## User Stories

1. As an operator, I want the app to open a local board running on my Mac without asking me anything, so that the first run needs no configuration.
2. As an operator, I want to see which board is open, by its address, so that I know what I am looking at.
3. As an operator, I want to see who I am on the open board, by my handle and name as the board knows them, so that I know as whom I would write.
4. As an operator with no board running, I want to be told how to start one, so that I am not left in front of an empty window.
5. As an operator, I want the app to notice a board I start after opening it, so that I do not have to restart the app.
6. As an operator, I want the app to notice when the open board stops, so that it never shows a board that is gone.
7. As an operator running several local boards, I want the app to open the one I started most recently, so that the board I just started is the one I see.
8. As an operator, I want a board left behind by a `vb serve` that was killed to be ignored, so that the app never opens a board that does not answer.
9. As an operator, I want the app to ignore a token file whose address is not on my own machine, so that a local token is never sent anywhere else.
10. As an operator, I want my board's token to stay inside the core and never reach the page, so that credentials live in one small process, as ADR 0002 requires.
11. As an operator, I want the app to make no connection other than to boards on my machine, with no remote fonts, scripts or update checks, so that ADR 0013 holds from the first build.
12. As an operator, I want the window to appear already drawn, with no white flash, so that the app feels instant from its first frame.
13. As an operator, I want the title bar to belong to the app, with the macOS window buttons inset into it, so that the window reads as a developer tool and not as a web page.
14. As an operator, I want to move the window by dragging its title bar, so that the custom title bar behaves like a native one.
15. As an operator, I want the app to follow my Mac's light or dark appearance, so that it matches the rest of my desktop.
16. As an operator, I want Geist, Geist Mono and Geist Pixel to render offline, so that the app looks the same with or without a connection.
17. As an operator, I want the installer to stay under 15 MB, so that getting the app is trivial.
18. As a future translator, I want every user-facing string in one typed English dictionary, so that adding a language means writing one more dictionary.
19. As a contributor, I want one command that runs the app in development with the UI reloading on save, so that I see a change the moment I make it.
20. As a contributor, I want one command that runs everything CI runs (formatting, lints, types, tests, conventions, the contract's freshness), so that I know before pushing whether CI will pass.
21. As a contributor, I want one command that builds the installable app, so that anyone can produce the same artifact CI measures.
22. As a contributor, I want the naming rule (one lowercase word per file and folder) checked by a machine, so that it does not depend on a reviewer noticing.
23. As a contributor, I want the names a tool dictates (`Cargo.toml`, `package.json`, `README.md`) accepted by that check, so that the rule's stated exceptions do not fail it.
24. As a contributor, I want comments found by a real lexer for each language, so that a `//` inside a string or a URL is never reported and a comment inside JSX always is.
25. As a contributor, I want `unwrap` and `expect` rejected outside tests, so that the core never panics on input it did not expect.
26. As a contributor, I want `any` rejected in TypeScript, so that types stay explicit at every boundary.
27. As a contributor, I want unsafe Rust forbidden, so that the core's memory safety is never in question.
28. As a contributor, I want the TypeScript types of the contract generated from the Rust types, so that the two sides cannot drift.
29. As a contributor, I want a check that fails when the generated types are stale, so that a change to the contract cannot be pushed without them.
30. As a contributor working on the UI, I want to run it in an ordinary browser against a fake core, so that I can work on it without building Rust or running a board.
31. As a contributor, I want the UI's tests to run against that same fake core, so that they test what a person sees and not how the core is reached.
32. As a contributor, I want the core's tests to run without Tauri, a webview or macOS, so that they are fast and run on any CI machine.
33. As a contributor, I want the core's tests to use a real HTTP server standing in for `vb serve`, so that they exercise the same requests a real board receives.
34. As a contributor, I want to know where a test file goes, so that test files keep one-word names too.
35. As a contributor, I want CI to fail when the UI's compressed JavaScript passes 250 KB, so that the budget is defended by a machine.
36. As a contributor, I want CI to fail when the installer passes 15 MB, for the same reason.
37. As a contributor, I want CI to print every measured budget even when it passes, so that a slow drift is visible before it breaks.
38. As a contributor, I want the Rust and bun versions pinned in the repository, so that a build today and a build next month use the same tools.
39. As a maintainer, I want the core never to depend on Tauri, so that the browser binary of ADR 0002 can reuse it unchanged.
40. As a maintainer, I want the design tokens defined once as CSS variables, with the accent derived from two values, so that the identity of ADR 0016 can change without touching a component.

## Implementation Decisions

### Workspace

- A Cargo workspace and a bun workspace share the repository root, laid out as ADR 0017 says: the `blossom` core library, the `desktop` Tauri binary, and the `ui` package.
- The workspace also gets a third, development-only crate for the conventions check. It is never shipped and never a dependency of the product. ADR 0017 gains a line saying so.
- Rust uses edition 2024 and the latest stable toolchain, installed through rustup. `rust-version` and the CI toolchain are pinned to it (1.100 on 2026-11-20) and raised together when a new stable ships. Crates follow their latest stable releases; Tauri stays on its 2.x line until 3.0 leaves alpha.
- Lints are declared once for the whole workspace: `unwrap_used` and `expect_used` denied, with Clippy's own settings allowing both in tests; `unsafe_code` forbidden; warnings fail CI.
- bun is the package manager and script runner, its version pinned in the root manifest. Vite, Vitest and Tauri's CLI run under it. The root holds the scripts and Tauri's CLI; `ui` is the only JavaScript package.
- Three commands cover the work: develop, check and build. `check` runs exactly what CI runs.

### Conventions check

- It walks the tracked files and reports every path segment that is not one lowercase word, with an optional extension, so `list.tsx` and `bun.lock` pass and `list.test.tsx` or `vite.config.ts` fail.
- Names a tool dictates and cannot be configured are listed explicitly (`Cargo.toml`, `Cargo.lock`, `package.json`, `README.md`, `LICENSE`, `CLAUDE.md`, `GLOSSARY.md`, `Tauri.toml`, and the like). Where a tool lets a name be configured, it is configured instead of listed: Vite's config file and Tauri's icon files get one-word names.
- Third-party skills under `.claude` and the tracker under `.scratch` are outside the check.
- It reports every comment in Rust, TypeScript and CSS. Each language is read by a real lexer: rustc's own lexer, as published for rust-analyzer, for Rust, and a TypeScript parser that understands JSX for TypeScript. CSS is simple enough for a small lexer of strings and comments. A regex is not acceptable.
- Generated files are configured to carry no header comment. If a generator cannot drop its header, its one output file is excluded by name.

### Core: finding local boards

- The core lists the boards `vb serve` is running from the folder it writes them to: `board/local` under `$XDG_CONFIG_HOME`, or under `~/.config` when that is unset. This is `~/.config` on macOS too, not the platform's configuration folder, because that is where `vb` looks.
- Each file there holds `{ url, token, handle }`. A file is a candidate only when its URL is on loopback (`127.0.0.1`, `localhost`, `[::1]`). Any other address is ignored, so no token leaves the machine.
- A candidate becomes a local board only when discovery answers at its URL and names that same URL, the rule `vb` itself enforces as `wrong_audience`. A file whose board does not answer, or answers for another URL, is ignored.
- For the board that is opened, the core calls `GET /me` with the token. The handle and name it shows come from that answer, not from the file. A token the board refuses makes the board unopenable, as if it were not running.
- The open board is the local board started most recently, judged by when its file was written. Choosing among several by hand is out of scope.
- The core notices files appearing and disappearing within about a second. Whether it watches the folder or polls it is decided in the ticket. The trigger is injectable, so tests never sleep.
- The folder's location is a parameter of the core, so tests point it at a temporary directory.
- The HTTP client runs on the tokio runtime that Tauri already brings. TLS stays off until a board that needs it is connected, which keeps the binary small. The client is chosen by weight, and the size it adds is recorded in the ticket.
- Errors are typed. A missing folder, an unreadable file, a board that does not answer and a refused token are all states the core handles; none of them panics.

### Contract

- The skeleton needs one subscription and no requests: the open board, pushed whenever it changes. Its value is either "no board" or an open board with its URL and the person's handle and name. The token is never part of it. The request half of ADR 0018 is built with the first request.
- The TypeScript types are generated from the Rust types into a single file. The generator, specta or ts-rs, is chosen in the ticket. Checking freshness means generating again and failing on any difference.
- The core offers the subscription without knowing about Tauri. The desktop shell forwards it to the page over a Tauri IPC channel.

### Desktop shell

- Tauri 2, configured in `Tauri.toml`, with the identifier `io.github.galfrevn.blossom` (ADR 0020) and the product name Blossom. macOS only.
- The window has an overlay title bar with its title hidden and the window buttons inset. It stays hidden until the UI has painted once, and is shown then.
- A content security policy lets the page load only its own bundled assets and talk only to the core over IPC, so ADR 0013 is enforced inside the webview itself.
- Capabilities grant only what the skeleton uses: showing the window, dragging it, and the open-board subscription. No Tauri plugins.

### UI

- Vite, React and TypeScript, with `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes` and `verbatimModuleSyntax`. Biome formats and lints, with `noExplicitAny` as an error.
- The UI reaches the core through one client module (ADR 0018). It has two implementations behind the same generated types: Tauri's, and a fake core used by the tests and by development in a browser.
- The open board lives outside React, in a small store fed by the subscription and read through `useSyncExternalStore`. No state library.
- Tailwind 4 is wired through its Vite plugin (ADR 0019). Its theme points at CSS variables for light and dark, switched by the system's appearance. The scaffold takes the values from ADR 0016 as they stand when it is built (OKLCH cream and ink, the accent from `--accent-hue` and `--accent-chroma`) and invents no palette of its own. The identity work owns those values.
- Geist, Geist Mono and Geist Pixel are bundled from the `geist` package through Vite. Their license, the SIL Open Font License, ships with the app.
- Every user-facing string goes through a typed English dictionary. A missing key is a type error.
- There are two screens and a frame: the frame is the title bar and the status bar; the screens are "no board running", with how to start one, and "board open", with its address and who you are.

### Tests and CI

- Rust unit tests live in the file they test. Integration tests live in each crate's own tests folder.
- TypeScript tests live in a `tests` folder beside the code they test (`attention/list.tsx` is tested by `attention/tests/list.tsx`), and Vitest is configured to find them there. `CLAUDE.md` gains this rule.
- GitHub Actions runs on every push and pull request. A Linux job runs the conventions check, the core's formatting, lints and tests, the UI's lint, types, tests and build, the contract's freshness, and the JavaScript budget. A macOS job lints and builds the desktop shell, builds the installer, and checks the installer budget.
- The JavaScript budget is the sum of the gzip sizes of the JavaScript files the UI build emits. The installer budget is the size of the disk image. Both are printed on every run.
- If the repository is private, macOS minutes cost ten times Linux minutes. The macOS job is kept to what needs macOS.

## Testing Decisions

- A good test drives the code through the seam a real caller uses and asserts what that caller sees. It survives a rewrite of the internals.
- **There is one seam: the contract (ADR 0018), tested from both sides.**
  - **Core side.** Tests drive the core through its open-board subscription. A temporary directory stands in for the token folder, and a small real HTTP server stands in for `vb serve`, answering discovery and `/me`. The scenarios: no folder; an empty folder; one board; a file whose board does not answer; a board whose discovery names another URL; a non-loopback URL; a refused token; two boards, where the newer opens; a board that starts while subscribed; the open board stopping.
  - **UI side.** Tests render the whole UI against the fake core, with Vitest and Testing Library, and assert what a person sees: how to start a board when there is none, the board's address and the person's name when one is open, and the change when the fake core pushes a new value.
- **The desktop shell** only forwards. It is tested through Tauri's mock runtime if that runtime can carry an IPC channel. Otherwise it is covered by a manual check of the running app against a real `vb serve`, recorded in the ticket.
- **The conventions check** is tested on small trees in temporary directories:
  - names that must fail: a hyphenated name, a two-word dotted name, an uppercase name;
  - names that must pass: a dictated name;
  - comments that must be reported: a Rust doc comment, a nested block comment, a comment inside JSX;
  - text that must not be reported: `//` inside a Rust string, a URL in JSX text.
- There is no prior art: this is the first code. The `tdd` skill's notes on tests and mocking apply. Mock only at the edges the project does not own, and prefer a real local server to a mocked HTTP client.

## Out of Scope

- The field to paste another board's address. Boards that are not local need the token work of the third spike, or a tailnet.
- Switching between boards, and remembering which one was open last.
- Any board content: channels, threads, the replica and sync. Those are the sync spike's.
- Noticing a board that dies without removing its file. That comes with the event stream.
- Vercel's hosted board, tailnet boards and TLS.
- Glass and window vibrancy (ADR 0006).
- The palette, the type scale, the mark and the wordmark (ADR 0016, owned by the identity work).
- A theme switch inside the app. It follows the system for now.
- Notifications, the Dock badge, the command palette and keyboard shortcuts.
- Measuring cold start, memory and scrolling. The list spike does that; CI checks only the two size budgets.
- Signing, notarization and distribution (ADR 0012).
- Windows, Linux, the browser binary and the WebSocket transport.

## Further Notes

- What `vb serve` writes and where was read from Board's source at commit `bff12dc`, recorded in `documentation/research/board.md`. Board is experimental and moves fast, so check it against the installed version when this is built.
- The visual identity is being defined in a parallel session (ADR 0016). The scaffold consumes its tokens and fonts. As of 2026-11-20 the color tokens are settled but live only in the identity draft, not in a file in the repository. They are OKLCH custom properties: neutrals and an accent derived from `--accent-hue` and `--accent-chroma`, light on `:root`, dark under `prefers-color-scheme` and `[data-theme="dark"]`, and a few tokens meant only for the mark. The UI imports that file as it is once it exists. It does not copy the values out of the draft.
- Two documents change along the way: ADR 0017 gains the development-only conventions crate, and `CLAUDE.md` gains the rule for where tests go.
