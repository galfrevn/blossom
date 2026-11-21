---
adr: 0013
status: accepted
date: 2026-11-20
---

# The app talks only to the board

The app sends no usage data and makes no connection other than to the board a person connected. That includes not reporting errors to an outside service. For an open source tool that handles a team's credentials, "it talks to nobody else" is worth more than the metrics would be.

## Consequences

- There are no analytics to learn from. What people use and where it breaks has to come from issues and from talking to them.
- Crash reports stay on the machine. If they are ever needed, the person sends them by hand.
- Anything that would phone home, such as an update check, needs its own decision and has to be visible to the person.
- The core's HTTP client follows no redirects and uses no proxy, not even one set in the environment, so a request meant for a board reaches that board and nothing else. A local process that answers like a board and redirects elsewhere is simply not opened.
- The page itself can reach only its own assets and the core: the desktop shell's content security policy enforces this.
- ADR 0022 amends this for Vercel's hosted boards. To get a hosted board's token, the app sends one request of its own, the exchange at `oidc.vercel.com`, about every 12 hours, and the `vercel` process it starts talks to Vercel as the person. Local boards are unchanged.
