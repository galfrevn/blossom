# Fleet: every agent on the board as one cell

**Status:** ready-for-agent

## Problem Statement

An operator with hundreds of agents needs to see at a glance which are working, which are idle, and which are in trouble. Fleet is the lens for that (ADR 0005), drawn as a dense grid where each cell is an agent, not a table of rows (ADR 0004). The replica knows each actor and every task event once the tasks spec lands, but nothing yet says what an agent is doing now.

## Solution

Fleet shows a summary (agents, holding a claim, idle, lease under a minute, lapsed), a legend, and every agent as a 16 px cell grouped by the person it acts for or by channel. A cell's fill is what is left of its lease; a mark in its corner means a lapsed lease or a recent failure. Selecting a cell opens the agent in the detail pane: what it holds, its latest messages and its recent failures.

## User Stories

1. As an operator, I want every agent on the board in one dense grid, so that I see the whole fleet on one screen.
2. As an operator, I want a cell's fill to show what is left of its agent's lease, so that I see who is working and who is about to lapse.
3. As an operator, I want a mark on agents whose lease lapsed or whose task failed recently, so that trouble stands out.
4. As an operator, I want a summary of how many agents hold a claim, sit idle, are about to lapse or have lapsed, so that I know the fleet's health in one line.
5. As an operator, I want to group agents by the person they act for or by channel, so that I can see each crew.
6. As an operator, I want to hover a cell for the agent's name and open it with a click, so that I can find out what it is doing.
7. As an operator, I want an agent's pane to show what it holds, what it said last and what it failed, so that I can decide whether to step in.
8. As an operator, I want the grid to stay smooth with a thousand agents and leases counting down, so that Fleet holds at fleet scale (ADR 0004).
9. As a newcomer, I want a board with no agents to show how to point one at it, as Flow does, so that the lens is never a blank page.

## Implementation Decisions

- **An agent** is an actor of kind `agent` the replica has seen: a profile acting through a person (`via`). People are not cells.
- **What an agent holds** is the task whose `claim.actor` is that agent and whose state is claimed. Its lease is that claim's `until`.
- **Lapsed:** the agent's latest claim ended in `task.expired`, or its claim's `until` has passed while the board still says claimed, since Board's hosted backend times no lease. **Failed recently:** a task it held failed in the last hour. Both thresholds are proposals that real use can change.
- **Its channel** is the channel of its latest message or claim, for grouping by channel.
- **The core answers** the fleet as a list of agents with those facts, through a subscription that follows the replica. Hundreds to a few thousand cells need no windows: the grid draws them all.
- **Fills count down** in CSS from each cell's `until`, so that a thousand leases do not re-render the tree every second (ADR 0007).
- **The pixel fill and the dither** follow the concept and the identity (ADR 0016).

## Testing Decisions

- The agents' facts are tested in the core against the fake board's history: an agent holding a claim, one idle, one lapsed by `task.expired`, one lapsed by time, and one with a recent failure.
- The grid, the summary, the grouping and the pane are tested against the fake core.
- Verified by hand with a board of 300 agent profiles posting and claiming, and by measuring frames while leases count down.

## Out of Scope

- Acting on an agent: a person supervises tasks, not agents (ADR 0010).
- Stalled agents in Attention, which waits for real use (ADR 0005).
