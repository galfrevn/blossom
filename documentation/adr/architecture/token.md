---
adr: 0022
status: accepted
date: 2026-11-20
---

# Blossom gets a hosted board's token through the Vercel CLI

Recommended by the token spike and accepted by the owner. The findings are in `documentation/research/board.md`, under "Minting a token for Vercel's hosted board".

To read Vercel's hosted board, Blossom needs a token whose audience is the board, minted with the person's Vercel login. Blossom mints it the way `vb` does and shares `vb`'s cache, but it leaves the Vercel login to the Vercel CLI: it never reads, refreshes or writes that login itself.

1. Blossom first reads Board's token cache, `~/.config/board/tokens/`, in Board's own format, and uses a token until five minutes before it expires. That sends no request.
2. When there is none, Blossom runs the Vercel CLI. `vercel api` checks that the board's host belongs to the project the board names, and `vercel project token` gives a development token. The CLI refreshes and stores its own login.
3. Blossom exchanges the development token at `https://oidc.vercel.com/~token` for the board's audience, and writes the result into Board's cache, so `vb` and Blossom mint once between them.
4. The token never reaches the replica, the page or a log.

## Consequences

- Opening a hosted board needs the Vercel CLI, installed and logged in. When it is missing or logged out, Blossom says so and names the command to run: `vercel login`.
- This amends ADR 0013 for hosted boards. Besides the board, the app itself sends one request, the exchange to `oidc.vercel.com`, about every 12 hours while a hosted board is open. The `vercel` process it starts talks to `api.vercel.com` and `vercel.com` as the person, the same as when they run it themselves.
- Minting takes a few `vercel` processes, a few seconds every 12 hours for each board.
- Blossom writes into a folder `vb` owns. It must keep `vb`'s file names, JSON and permissions exactly, and follow them when Board changes them.

## Considered options

- **Reading the Vercel CLI's `auth.json` and calling Vercel's API directly.** Rejected. The access token in it is usually expired: on the owner's machine, 94 hours past. Refreshing it means posing as the Vercel CLI's OAuth client and rotating the refresh token the CLI also holds. If both refreshed at once, one would be left with a dead token, and a failed refresh in `@vercel/oidc` logs the person out. A login kept in the Keychain cannot be read without macOS asking the person.
- **Blossom's own "Sign in with Vercel".** It would leave the Vercel CLI out entirely, with Blossom's own refresh token in the Keychain. It needs an OAuth application registered with Vercel and owned by the project, and a browser sign-in. Revisit it if people without the Vercel CLI ask for hosted boards.
- **A token pasted by hand.** Rejected as the way in, because the token lasts 12 hours.
