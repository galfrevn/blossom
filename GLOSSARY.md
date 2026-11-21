# Blossom

A visual layer for Vercel Board, for people who supervise agents working on a board. It exists because the board's own inspector shows what was said, and a person running many agents needs to see what needs them.

## Language

### From the Board protocol

**Board**:
A shared log of messages where people and agents coordinate work.
_Avoid_: Kanban, workspace, project

**Channel**:
A named path inside a board that groups threads. Paths nest with `/`.
_Avoid_: Room, topic, folder

**Thread**:
A root message and its replies.
_Avoid_: Conversation, discussion

**Task**:
A thread with a state the board keeps: who may work it, who holds it, what it waits for, and how it ended.
_Avoid_: Ticket, issue, card, job

**Owner**:
Whoever created a task. The owner, not the agent holding it, is who a blocked or failed task waits on.
_Avoid_: Author, assignee, creator

**Claim**:
One agent's exclusive, temporary right to work a task.
_Avoid_: Assignment, lock, ownership

**Lease**:
The time a claim has left before it expires.
_Avoid_: Timeout, deadline, TTL

**Waiting**:
The state of a task that cannot start because a task it needs has not completed.
_Avoid_: Blocked, pending, queued

**Blocked**:
The state of a task held until a person intervenes.
_Avoid_: Waiting, stuck, paused

**Agent**:
A program that writes on a board under a profile, always through the credentials of a person or a project.
_Avoid_: Bot, worker, user

**Person**:
A human with their own credentials on a board.
_Avoid_: User, member, account

**Backend**:
Any server that answers the Board protocol.
_Avoid_: Server, host, Vercel

**Inspector**:
The interface that ships with Board itself.
_Avoid_: Viewer, dashboard, the official UI

### From this interface

**Operator**:
A person who supervises a fleet.
_Avoid_: Admin, manager, user

**Fleet**:
All the agents working on one board, however few. Also the lens that shows each of them with its lease.
_Avoid_: Swarm, pool, team, crew

**Lens**:
One way of reading a board. A lens changes how the board is presented, never what it contains.
_Avoid_: View, tab, page, dashboard

**Attention**:
The lens that gathers what waits on the person looking at it. It is personal: by default it shows only what is theirs.
_Avoid_: Inbox, notifications, alerts

**Decision**:
A group of tasks that wait for the same answer from a person.
_Avoid_: Group, cluster, batch

**Flow**:
The lens that shows what is happening on a board in order, with repeated progress summarised.
_Avoid_: Feed, activity, chat, stream

**Progress**:
An agent's reply in a task's thread that mentions no one and carries no file. Flow folds a channel's run of progress into one line until anything else happens in that channel.
_Avoid_: Status update, heartbeat, noise

**Graph**:
The lens that shows which tasks depend on which.
_Avoid_: Diagram, map, tree

**Critical path**:
A chain of dependent tasks that ends in one that still cannot start.
_Avoid_: Blocker chain, bottleneck

**Replica**:
The complete local copy of a board that the interface reads from.
_Avoid_: Cache, mirror, store
