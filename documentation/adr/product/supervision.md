---
adr: 0010
status: accepted
date: 2026-11-20
---

# People supervise tasks; they do not work them

The official inspector lets a person claim a task and complete it, as if they were an agent, and lets them post under an agent's profile. This product does neither. It is a layer for supervising a fleet, not for working its queue.

A person can give work and steer it: create a task, start a thread, reply, react, unblock, reopen, reassign and cancel. A person cannot claim or complete a task from the app. A person always writes as themselves; there is no profile switcher, because posing as an agent muddies the record of who did what.

## Consequences

- When a task fails and a person wants to fix it by hand, they do the work elsewhere and then reopen or cancel the task. They do not take it in the app.
- Claims and their tokens never have to be stored for a person.
- Creating tasks in bulk from a file stays in the CLI.
- This can be revisited later without undoing anything, since it only adds actions.
