import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { createClock } from "../../core/clock";
import type {
  FlowChange,
  TaskChange,
  TaskRow,
  Thread,
} from "../../core/contract";
import { createFakeCore } from "../../core/fake";
import { dictionary } from "../../dictionary";
import { createThreadOpenings } from "../../thread/openings";
import { TasksLens } from "../lens";

afterEach(cleanup);

const ONE_SECOND = 1000;
const NAMES = dictionary.tasks.stateNames;

function taskRow(overrides: Partial<TaskRow> & { id: string }): TaskRow {
  return {
    state: "ready",
    title: `task ${overrides.id}`,
    channel: "fleet/www",
    owner: "ada",
    holder: null,
    lease: null,
    changedAt: "2026-11-21T10:27:00.000Z",
    ...overrides,
  };
}

function threadOfTask(row: TaskRow): Thread {
  const root = {
    id: row.id,
    author: row.owner,
    onBehalfOf: null,
    createdAt: row.changedAt,
    text: row.title,
    edited: false,
    deleted: false,
  };
  return { id: `thread-${row.id}`, channel: row.channel, messages: [root] };
}

const nothing = () => undefined;

function renderTasksLensOf(rows: TaskRow[]) {
  const fakeCore = createFakeCore();
  let taskChange: TaskChange | undefined;
  let flowChange: FlowChange | undefined;
  fakeCore.core.subscribeToTasks((change) => {
    taskChange = change;
  });
  fakeCore.core.subscribeToFlow((change) => {
    flowChange = change;
  });
  fakeCore.pushTasks(rows);
  fakeCore.pushThreads(rows.map(threadOfTask));
  const clock = createClock(ONE_SECOND);
  const threadOpenings = createThreadOpenings();
  const lens = (channel: string | null) => (
    <TasksLens
      change={taskChange}
      flowChange={flowChange}
      channel={channel}
      clock={clock}
      onWiden={nothing}
      readTasks={fakeCore.core.readTasks}
      readTaskCounts={fakeCore.core.readTaskCounts}
      readThread={fakeCore.core.readThread}
      openLink={fakeCore.core.openLink}
      threadOpenings={threadOpenings}
    />
  );
  const { rerender } = render(lens(null));
  let channelNow: string | null = null;
  return {
    threadOpenings,
    pushTasks: (newRows: TaskRow[]) => {
      fakeCore.pushTasks(newRows);
      act(() => {
        rerender(lens(channelNow));
      });
    },
    narrowTo: (channel: string | null) => {
      channelNow = channel;
      act(() => {
        rerender(lens(channel));
      });
    },
  };
}

function taskList() {
  return screen.getByRole("list", { name: dictionary.tasks.list });
}

function drawnTaskIds(): (string | undefined)[] {
  return within(taskList())
    .queryAllByRole("listitem")
    .map((row) => row.dataset.task);
}

async function rowOf(id: string): Promise<HTMLElement> {
  const title = await within(taskList()).findByText(`task ${id}`);
  const row = title.closest<HTMLElement>("li");
  if (row === null) {
    throw new Error(`the list draws no row for task ${id}`);
  }
  return row;
}

function chips() {
  return within(screen.getByRole("group", { name: dictionary.tasks.states }));
}

function chip(name: string) {
  return chips().findByRole("button", { name });
}

function typeIntoTheFilter(title: string) {
  fireEvent.change(
    screen.getByRole("searchbox", { name: dictionary.tasks.filter }),
    { target: { value: title } },
  );
}

test("the lens says what it shows", () => {
  renderTasksLensOf([]);

  expect(
    screen.getByRole("heading", { name: dictionary.lenses.tasks.title }),
  ).toBeDefined();
  expect(screen.getByText(dictionary.lenses.tasks.description)).toBeDefined();
});

test("a chip for each state with tasks says how many it has", async () => {
  renderTasksLensOf([
    taskRow({ id: "1", state: "blocked" }),
    taskRow({ id: "2", state: "ready" }),
    taskRow({ id: "3", state: "ready" }),
  ]);

  const blocked = await chip(`${NAMES.blocked} 1`);

  expect(blocked.getAttribute("aria-pressed")).toBe("false");
  expect(await chip(`${NAMES.ready} 2`)).toBeDefined();
  expect(chips().getAllByRole("button")).toHaveLength(2);
});

test("choosing a chip narrows the list to its state, and choosing it again widens it", async () => {
  renderTasksLensOf([
    taskRow({ id: "1", state: "blocked" }),
    taskRow({ id: "2", state: "ready" }),
  ]);
  const blocked = await chip(`${NAMES.blocked} 1`);

  act(() => {
    blocked.click();
  });
  await waitFor(() => expect(drawnTaskIds()).toEqual(["1"]));
  const pressed = blocked.getAttribute("aria-pressed");
  act(() => {
    blocked.click();
  });

  expect(pressed).toBe("true");
  await waitFor(() => expect(drawnTaskIds()).toEqual(["1", "2"]));
});

test("the sidebar's channel narrows the list and the chips' counts", async () => {
  const lens = renderTasksLensOf([
    taskRow({ id: "1", channel: "fleet/www" }),
    taskRow({ id: "2", channel: "fleet/api" }),
    taskRow({ id: "3", state: "failed", channel: "general" }),
  ]);
  await rowOf("3");

  lens.narrowTo("fleet");

  await waitFor(() => expect(drawnTaskIds()).toEqual(["1", "2"]));
  expect(await chip(`${NAMES.ready} 2`)).toBeDefined();
  expect(
    chips().queryByRole("button", { name: `${NAMES.failed} 1` }),
  ).toBeNull();
});

test("a chosen state keeps its chip while the channel holds none of its tasks", async () => {
  const lens = renderTasksLensOf([
    taskRow({ id: "1", state: "failed", channel: "general" }),
    taskRow({ id: "2", channel: "fleet/www" }),
  ]);
  const failedEverywhere = await chip(`${NAMES.failed} 1`);
  act(() => {
    failedEverywhere.click();
  });

  lens.narrowTo("fleet");

  const failedHere = await chip(`${NAMES.failed} 0`);
  expect(failedHere.getAttribute("aria-pressed")).toBe("true");
});

test("the box narrows the list by title, whatever its case", async () => {
  renderTasksLensOf([
    taskRow({ id: "1", title: "Fix CVE-123 in acme/web" }),
    taskRow({ id: "2", title: "deploy acme/api" }),
  ]);
  await within(taskList()).findByText("deploy acme/api");

  typeIntoTheFilter("cve");

  await waitFor(() => expect(drawnTaskIds()).toEqual(["1"]));
});

test("a task that changes on the board changes in the list and in the chips", async () => {
  const lens = renderTasksLensOf([taskRow({ id: "1" })]);
  await chip(`${NAMES.ready} 1`);

  lens.pushTasks([taskRow({ id: "1", state: "failed" })]);

  expect(await chip(`${NAMES.failed} 1`)).toBeDefined();
  expect(await within(await rowOf("1")).findByText(NAMES.failed)).toBeDefined();
});

test("selecting a row opens the task's thread in the pane and marks the row", async () => {
  renderTasksLensOf([
    taskRow({ id: "1" }),
    taskRow({ id: "812", channel: "fleet/api" }),
  ]);
  const row = await rowOf("812");

  act(() => {
    row.click();
  });

  const pane = await screen.findByRole("complementary", {
    name: dictionary.thread.inChannel("fleet/api"),
  });
  expect(await within(pane).findByText("task 812")).toBeDefined();
  expect(row.getAttribute("aria-current")).toBe("true");
});

test("a thread opened from elsewhere opens in the pane and marks its row", async () => {
  const lens = renderTasksLensOf([
    taskRow({ id: "1" }),
    taskRow({ id: "812", channel: "fleet/api" }),
  ]);
  const row = await rowOf("812");

  act(() => {
    lens.threadOpenings.open("812");
  });

  const pane = await screen.findByRole("complementary", {
    name: dictionary.thread.inChannel("fleet/api"),
  });
  expect(await within(pane).findByText("task 812")).toBeDefined();
  expect(row.getAttribute("aria-current")).toBe("true");
});
