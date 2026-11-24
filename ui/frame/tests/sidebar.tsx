import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import type { TaskChange, TaskRow } from "../../core/contract";
import { createFakeCore } from "../../core/fake";
import type { Lens } from "../choice";
import { Sidebar } from "../sidebar";

afterEach(cleanup);

const nothing = () => undefined;

const countingNothing = {
  taskChange: undefined,
  readTaskCounts: () => Promise.resolve([]),
};

function renderSidebar(
  channels: string[],
  chosenChannel: string | null = null,
  chosenLens: Lens = "flow",
) {
  return render(
    <Sidebar
      channels={channels}
      chosenChannel={chosenChannel}
      chosenLens={chosenLens}
      onChooseChannel={nothing}
      onChooseLens={nothing}
      {...countingNothing}
    />,
  );
}

function lensButton(name: string) {
  const lenses = within(screen.getByRole("list", { name: "Lenses" }));
  return lenses.getByRole("button", { name: new RegExp(`^${name}`, "u") });
}

function channelRows() {
  const channels = within(screen.getByRole("list", { name: "Channels" }));
  return channels.getAllByRole("listitem");
}

test("the sidebar lists the lenses, with the chosen one as the current page", () => {
  renderSidebar([], null, "tasks");

  expect(lensButton("Flow").getAttribute("aria-current")).toBe("false");
  expect(lensButton("Tasks").getAttribute("aria-current")).toBe("page");
});

test("each lens says the key that chooses it", () => {
  renderSidebar([]);

  expect(lensButton("Flow").getAttribute("aria-keyshortcuts")).toBe("2");
  expect(lensButton("Tasks").getAttribute("aria-keyshortcuts")).toBe("3");
});

test("the sidebar lists the board's channels as a tree", () => {
  renderSidebar(["fleet/api", "fleet/www", "general"]);

  const rows = channelRows().map((row) => ({
    name: row.textContent,
    depth: row.getAttribute("data-depth"),
  }));

  expect(rows).toEqual([
    { name: "fleet", depth: "0" },
    { name: "#api", depth: "1" },
    { name: "#www", depth: "1" },
    { name: "#general", depth: "0" },
  ]);
});

test("a channel that also holds channels is listed once, as a channel", () => {
  renderSidebar(["fleet", "fleet/www"]);

  expect(channelRows().map((row) => row.textContent)).toEqual([
    "#fleet",
    "#www",
  ]);
});

test("the sidebar follows the board's channels as they appear", () => {
  const { rerender } = renderSidebar(["general"]);

  rerender(
    <Sidebar
      channels={["fleet/docs", "general"]}
      chosenChannel={null}
      chosenLens="flow"
      onChooseChannel={nothing}
      onChooseLens={nothing}
      {...countingNothing}
    />,
  );

  expect(screen.getByText("#docs")).toBeDefined();
});

test("channels are listed in natural order, so area2 comes before area10", () => {
  renderSidebar(["fleet/area1", "fleet/area10", "fleet/area2"]);

  expect(channelRows().map((row) => row.textContent)).toEqual([
    "fleet",
    "#area1",
    "#area2",
    "#area10",
  ]);
});

test("choosing a channel, or a folder of channels, asks the lens to narrow to it", () => {
  const onChooseChannel = vi.fn();
  render(
    <Sidebar
      channels={["fleet/www", "general"]}
      chosenChannel={null}
      chosenLens="flow"
      onChooseChannel={onChooseChannel}
      onChooseLens={nothing}
      {...countingNothing}
    />,
  );

  screen.getByRole("button", { name: "#www" }).click();
  screen.getByRole("button", { name: "fleet" }).click();

  expect(onChooseChannel.mock.calls).toEqual([["fleet/www"], ["fleet"]]);
});

test("the chosen channel is marked, and the chosen lens stays the current page", () => {
  renderSidebar(["fleet/www", "general"], "fleet/www");

  expect(
    screen.getByRole("button", { name: "#www" }).getAttribute("aria-current"),
  ).toBe("true");
  expect(lensButton("Flow").getAttribute("aria-current")).toBe("page");
});

test("choosing a lens by a click asks to show it", () => {
  const onChooseLens = vi.fn();
  render(
    <Sidebar
      channels={["general"]}
      chosenChannel="general"
      chosenLens="flow"
      onChooseChannel={nothing}
      onChooseLens={onChooseLens}
      {...countingNothing}
    />,
  );

  lensButton("Tasks").click();

  expect(onChooseLens.mock.calls).toEqual([["tasks"]]);
});

function task(id: string, state: string, channel = "fleet/www"): TaskRow {
  return {
    id,
    state,
    title: `task ${id}`,
    channel,
    owner: "ada",
    holder: null,
    lease: null,
    changedAt: "2026-11-21T10:27:00.000Z",
  };
}

function renderSidebarCounting(rows: TaskRow[]) {
  const fakeCore = createFakeCore();
  let taskChange: TaskChange | undefined;
  fakeCore.core.subscribeToTasks((change) => {
    taskChange = change;
  });
  fakeCore.pushTasks(rows);
  const sidebar = (channel: string | null) => (
    <Sidebar
      channels={["fleet/api", "fleet/www", "general"]}
      chosenChannel={channel}
      chosenLens="flow"
      onChooseChannel={nothing}
      onChooseLens={nothing}
      taskChange={taskChange}
      readTaskCounts={fakeCore.core.readTaskCounts}
    />
  );
  const { rerender } = render(sidebar(null));
  let channelNow: string | null = null;
  return {
    pushTasks: (newRows: TaskRow[]) => {
      fakeCore.pushTasks(newRows);
      rerender(sidebar(channelNow));
    },
    choose: (channel: string | null) => {
      channelNow = channel;
      rerender(sidebar(channel));
    },
  };
}

function tasksCounted(description: string) {
  return screen.findByRole("button", { name: "Tasks", description });
}

test("beside Tasks, the sidebar counts the open tasks: blocked, failed, claimed, ready and waiting", async () => {
  const states = ["blocked", "failed", "claimed", "ready", "ready", "waiting"];
  const closed = ["completed", "completed", "canceled", "reviewing"];
  renderSidebarCounting(
    [...states, ...closed].map((state, index) => task(`${index}`, state)),
  );

  const tasks = await tasksCounted("6 open tasks");

  expect(within(tasks).getByText("6")).toBeDefined();
  expect(lensButton("Flow").textContent).toBe("Flow2");
});

test("the count is narrowed to the chosen channel and the channels under it", async () => {
  const sidebar = renderSidebarCounting([
    task("1", "ready", "fleet/www"),
    task("2", "blocked", "fleet/api"),
    task("3", "completed", "fleet/api"),
    task("4", "failed", "general"),
  ]);
  await tasksCounted("3 open tasks");

  sidebar.choose("fleet");
  await tasksCounted("2 open tasks");
  sidebar.choose("general");

  expect(await tasksCounted("1 open task")).toBeDefined();
});

test("the count follows the replica, down to none", async () => {
  const sidebar = renderSidebarCounting([task("1", "ready")]);
  await tasksCounted("1 open task");

  sidebar.pushTasks([task("1", "claimed"), task("2", "ready")]);
  await tasksCounted("2 open tasks");
  sidebar.pushTasks([task("1", "completed"), task("2", "canceled")]);

  const tasks = await tasksCounted("0 open tasks");
  expect(within(tasks).getByText("0")).toBeDefined();
});

test("until the core has counted, Tasks shows no count", () => {
  renderSidebar([]);

  expect(lensButton("Tasks").textContent).toBe("Tasks3");
});

test("the count sits between the lens's name and its key, so the key keeps its place", async () => {
  renderSidebarCounting([task("1", "ready")]);
  const count = within(await tasksCounted("1 open task")).getByText("1");

  expect(count.previousElementSibling?.textContent).toBe("Tasks");
  expect(count.parentElement?.lastElementChild?.tagName).toBe("KBD");
});
