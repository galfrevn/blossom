# Boards: several boards, one open at a time, readable offline

**Status:** ready-for-agent

## Problem Statement

The app opens the newest local board it finds and nothing else. ADR 0012 promises one board at a time with a switcher between the boards a person has connected, a field on the first run to paste the address of another, and reading offline: everything readable and searchable from the replica without a connection. Today a board that stops disappears from the app, though its replica stays on disk (sync spec, ticket 04), and a board shared on a tailnet cannot be opened at all.

## Solution

The title bar's board name becomes a switcher. It lists the boards the app knows: those with a replica on this Mac, and the local ones running now, each saying whether it is live or offline and when it was last seen. Choosing one opens it; one board is open at a time. A board that is gone stays readable from its replica, marked offline, with writing disabled, and goes live again when it answers. The start screen and the switcher take a pasted address, which opens a board shared on a tailnet. A board can be forgotten, which deletes its replica.

## User Stories

1. As an operator, I want to switch between the boards I have opened before, so that I can follow several projects.
2. As an operator, I want each board in the switcher to say whether it is live or offline and when it was last seen, so that I know what I am looking at.
3. As an operator, I want a board that stopped to stay readable and searchable from its replica, so that I can read it offline (ADR 0012).
4. As an operator, I want an offline board marked as such, with writing disabled, so that I never think it is live.
5. As an operator, I want an offline board to go live again by itself when it answers, so that I don't reconnect by hand.
6. As an operator, I want to paste the address of a board shared on a tailnet and open it, so that I can supervise a board that runs on another machine.
7. As an operator, I want the first run to offer that field when no local board is running, so that a newcomer with a remote board is not stuck (ADR 0012).
8. As an operator, I want to forget a board, deleting its replica, so that I control what stays on my Mac (ADR 0013).
9. As an operator, I want the app to open the board I had open last, so that a restart puts me back where I was.
10. As a contributor, I want switching, offline reading and pasted addresses tested in the core against fake boards, so that the boards' life cycle is right.

## Implementation Decisions

- **Known boards** are the replicas on disk, each with its address, identity, person and the time it was last live, kept in the replica. Running local boards come from the token folder, as today.
- **One open board.** Opening another stops keeping the first in step and keeps its replica. The follower of the token folder keeps following, so a newly started local board still shows in the switcher; it opens on its own only when no board is open.
- **Offline:** when the open board is gone (sync spec, ticket 04), the app keeps showing its replica, read-only, with the status bar saying "Offline · last seen 14:02". The core keeps asking it, at the pacing's slowest, and goes live when it answers, resuming from the stored cursor.
- **What the app opens on:** the board that was open when it quit, live or offline; else the newest running local board; else the start screen, which lists known boards to read offline and takes an address. This settles what the sync spec left out of scope.
- **A pasted address** is checked with discovery and `/me`, as a local board is. A board on a tailnet answers `/me` with the person's Tailscale login and needs no token (Board's tailnet guide), so the core's access gains a form without a token. An address that answers neither is refused with the reason. Hosted boards need a token and belong to the hosted spec.
- **Forgetting** asks for confirmation, deletes the replica and its attachments, and removes the board from the switcher. A running local board shows up again.
- **Privacy:** the app only talks to boards the person opened, found in the token folder, or pasted (ADR 0013).

## Testing Decisions

- The known boards, switching, offline and back, and forgetting are tested in the core against fake boards and real replicas in a temporary folder.
- A pasted address is tested against a fake board without a token, as a tailnet board answers, and against one that refuses.
- The switcher, the offline marks and the start screen are tested against the fake core.
- Verified by hand with two `vb serve` boards on two ports, one stopped and started again, and with a board behind `tailscale serve` if a tailnet is at hand.

## Out of Scope

- Several boards mixed in one view (ADR 0012).
- Vercel's hosted board: the hosted spec.
- Boards that need a token handed over by hand.
