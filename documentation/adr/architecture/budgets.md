---
adr: 0009
status: accepted
date: 2026-11-20
---

# Weight and speed budgets that every change has to respect

The software has to weigh almost nothing and be very fast and light. To make that defensible when someone wants to add a dependency or a feature, it is fixed as numbers.

| Budget | Target |
| --- | --- |
| Desktop installer | under 15 MB |
| Idle memory, with a large board loaded | under 150 MB |
| The UI's JavaScript, compressed | under 250 KB |
| Cold start to a usable screen | under 300 ms |
| Any interaction | under 50 ms |
| A list of 100,000 messages | scrolls at 120 fps |

These are targets, not measurements. Nothing has been measured yet, and the first technical spikes exist to find out whether they hold.

## Consequences

- A change that breaks a budget needs a reason written down, or it does not go in.
- The budgets should be checked automatically once there is something to build, so they do not erode quietly.
- If a spike shows a number is unrealistic, the number is revised here rather than ignored.
