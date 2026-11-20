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

## Clients that run agents

On 2026-11-20 Board's creators asked in public to see Board clients. One reply described an "ADE", an agent development environment: "runs agents on my machine, but uses board so that i can organize stuff in channels + agents instead of sessions". Its screenshot shows:

- A sidebar with every channel and, nested under each one, all of its threads, with reply counts and unread dots. Below them, the agents (one, `main`) and the person ("jb, on this machine").
- A channel page with a composer to start a thread and a list of recent threads.
- A person talking to one agent the way they would talk to a colleague: "hey there", "anyone here?", "@main do u listen to this channel". A `self-mod` channel where the agent changes the app itself.

It is an orchestrator with a chat client on top: the category ADR 0001 leaves out, with the chat-first default ADR 0005 rejects.

### What it tells us

- **It closes the loop and Blossom does not.** A message there reaches an agent the app itself runs. In Blossom, a task that no agent listens for stays untouched, because the app does not launch agents. A board that no agent has written on yet needs an empty state that shows how to point one at it.
- **It is the first sign of what early users do.** One local board, one agent, used mostly to talk to it. That is the first contact ADR 0004 describes, and for that person Attention opens empty while their conversation sits in Flow. ADR 0004 accepted that cost; this is the first evidence of how large it is. It is not enough to change the decision, and real use should settle it.
- **It does not scale, and it does not compete.** With one agent, a single channel already lists eight threads in the sidebar; with a hundred agents the sidebar becomes the feed ADR 0005 rejects. Its agents write to a board, so Blossom can open that same board and supervise them.
- **Minimal is now the default look.** Dashed dividers, tracked uppercase labels, square initials and one accent colour: the Geist look an agent produces when asked for an interface, and close to Blossom's. What sets Blossom apart is the pixel, dither and blossom of its identity (ADR 0016) and how it organises information, not its minimalism.

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
- The reply describing the ADE, with its screenshot, was shared in a session on 2026-11-20. Its link was not kept.
