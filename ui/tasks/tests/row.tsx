import { act, cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { createClock } from "../../core/clock";
import type { TaskRow } from "../../core/contract";
import { dictionary } from "../../dictionary";
import { TaskLine } from "../row";

const NOW = Date.parse("2026-11-21T10:30:00.000Z");
const ONE_SECOND = 1000;
const SECONDS_IN_AN_HOUR = 3600;
const AN_HOUR = SECONDS_IN_AN_HOUR * ONE_SECOND;
const SECONDS_LEFT = 65;
const NAMES = dictionary.tasks.stateNames;

function taskRow(overrides: Partial<TaskRow>): TaskRow {
  return {
    id: "812",
    state: "ready",
    title: "fix CVE-123 in acme/web",
    channel: "fleet/www",
    owner: "ada",
    holder: null,
    lease: null,
    changedAt: "2026-11-21T10:27:00.000Z",
    ...overrides,
  };
}

function renderLine(row: TaskRow) {
  render(
    <ol>
      <TaskLine
        index={0}
        setSize={1}
        row={row}
        selected={false}
        clock={createClock(ONE_SECOND)}
      />
    </ol>,
  );
  return within(screen.getByRole("listitem"));
}

beforeEach(() => {
  vi.useFakeTimers({ toFake: ["setInterval", "clearInterval", "Date"] });
  vi.setSystemTime(NOW);
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
});

test("a row shows the task's state, title, channel, owner and when it last changed", () => {
  const line = renderLine(taskRow({}));

  expect(line.getByText(NAMES.ready)).toBeDefined();
  expect(line.getByText("fix CVE-123 in acme/web")).toBeDefined();
  expect(line.getByText("#fleet/www")).toBeDefined();
  expect(line.getByText("ada")).toBeDefined();
  expect(line.getByText("3 min")).toBeDefined();
});

test("a claimed task's row shows who holds it and the lease it has left", () => {
  const line = renderLine(
    taskRow({
      state: "claimed",
      holder: "claude",
      lease: {
        since: null,
        until: new Date(NOW + SECONDS_LEFT * ONE_SECOND).toISOString(),
      },
    }),
  );

  expect(line.getByText(NAMES.claimed)).toBeDefined();
  expect(line.getByText("claude")).toBeDefined();
  expect(line.getByText("1:05")).toBeDefined();
});

test("a blocked task's row names who blocked it, with no lease", () => {
  const line = renderLine(taskRow({ state: "blocked", holder: "codex" }));

  expect(line.getByText("codex")).toBeDefined();
  expect(line.queryByRole("meter")).toBeNull();
});

test("a state the lens does not know reads as the board names it", () => {
  const line = renderLine(taskRow({ state: "reviewing" }));

  expect(line.getByText("reviewing")).toBeDefined();
});

test("when a task last changed follows the time", () => {
  const line = renderLine(taskRow({}));

  act(() => {
    vi.advanceTimersByTime(AN_HOUR);
  });

  expect(line.getByText("1 h 03 min")).toBeDefined();
});

test("a task that last changed days ago says how many", () => {
  const line = renderLine(taskRow({ changedAt: "2026-11-18T09:00:00.000Z" }));

  expect(line.getByText("3 d")).toBeDefined();
});
