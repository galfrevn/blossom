# Landscape

Who else is building around Board, and the neighbouring market, as of 2026-11-20.

## Vercel's position on UI

Board ships without a hosted interface on purpose. Chris Tate of Vercel put it this way at launch: "Everyone visualizes work differently: chat, boards, a terminal. Ask your agent to build yours."

Two things follow. Vercel is unlikely to compete hard on interface. And anyone can ask an agent for a basic UI, so the bar for standing out is quality.

## Community UIs for Board

A GitHub search one day after launch found seven repositories built on Board. All are thin web wrappers around the SDK or small backends:

- `bob-collective/board`: the official viewer served from your own address behind Google sign-in, with an installer for agents. It describes itself as about 300 lines of code.
- `recoupable/board` and `xdjs/MusicNerdBoard`: read-only or basic web views of one team's board.
- `Melbourneandrew/public-board`: a board with open registration.
- `andrew-bierman/board-beads` and `ivolivares/local-multi-agent-board`: alternative backends, not interfaces.
- `gscode1/boardhand`: workers that run tasks from a self-hosted board.

None of them keeps a local replica, searches, draws dependencies or shows a fleet.

## Other boards for agents

TalkTo was posted to Hacker News the same week as "Vercel Board, but focused solely on distributed agent messaging", deployed on Cloudflare. It is a competing backend, not an interface.

## The neighbouring market: agent orchestrators

Desktop and web apps that run coding agents (worktrees, terminals, kanban boards that launch agents) are a saturated category. The curated list `andyrewlee/awesome-agent-orchestrators` has about 130 entries under "Parallel Coding Agents, Desktop and Web" alone. Vibe Kanban, one of the best known, announced it is sunsetting and moving to community maintenance.

This project is not in that category. It does not run agents. It is a client for a protocol, the way a mail client is a client for mail.

## Sources

- https://github.com/bob-collective/board
- https://github.com/andyrewlee/awesome-agent-orchestrators
- https://news.ycombinator.com/item?id=50574220
- https://www.vibekanban.com/
