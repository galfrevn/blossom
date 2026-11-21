import { act, cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { Application } from "../application";
import type { OpenBoard, ReplicaState } from "../core/contract";
import { createFakeCore } from "../core/fake";
import { createStore } from "../core/store";

afterEach(cleanup);

const boardOfAda: OpenBoard = {
  kind: "open",
  url: "http://127.0.0.1:4100/api/board",
  person: { handle: "ada", name: "Ada Lovelace" },
};

function renderApplicationWhileTheCoreSays(said: {
  openBoard?: OpenBoard;
  replicaState?: ReplicaState;
  channels?: string[];
}) {
  const fakeCore = createFakeCore();
  const openBoardStore = createStore(fakeCore.core.subscribeToOpenBoard);
  const replicaStore = createStore(fakeCore.core.subscribeToReplica);
  const channelsStore = createStore(fakeCore.core.subscribeToChannels);
  if (said.openBoard !== undefined) {
    fakeCore.pushOpenBoard(said.openBoard);
  }
  if (said.replicaState !== undefined) {
    fakeCore.pushReplicaState(said.replicaState);
  }
  if (said.channels !== undefined) {
    fakeCore.pushChannels(said.channels);
  }
  render(
    <Application
      openBoardStore={openBoardStore}
      replicaStore={replicaStore}
      channelsStore={channelsStore}
    />,
  );
  return fakeCore;
}

function sidebar() {
  return within(screen.getByRole("complementary"));
}

function statusBar() {
  return within(screen.getByRole("contentinfo"));
}

test("the window is dragged by its title bar", () => {
  renderApplicationWhileTheCoreSays({ openBoard: { kind: "noneRunning" } });

  const titleBar = screen.getByRole("banner");

  expect(titleBar.hasAttribute("data-tauri-drag-region")).toBe(true);
});

test("with no board running, it shows how to start one", () => {
  renderApplicationWhileTheCoreSays({ openBoard: { kind: "noneRunning" } });

  expect(
    screen.getByRole("heading", { name: "No board running" }),
  ).toBeDefined();
  expect(screen.getByText("vb serve")).toBeDefined();
});

test("with a board open, the status bar shows its address and who you are on it", () => {
  renderApplicationWhileTheCoreSays({ openBoard: boardOfAda });

  expect(
    statusBar().getByText("http://127.0.0.1:4100/api/board"),
  ).toBeDefined();
  expect(statusBar().getByText("Ada Lovelace")).toBeDefined();
  expect(statusBar().getByText("@ada")).toBeDefined();
  expect(
    screen.queryByRole("heading", { name: "No board running" }),
  ).toBeNull();
});

test("until the core answers, it shows neither screen", () => {
  renderApplicationWhileTheCoreSays({});

  expect(
    screen.queryByRole("heading", { name: "No board running" }),
  ).toBeNull();
  expect(screen.queryByRole("contentinfo")).toBeNull();
});

test("the screen follows the core as a board starts and stops", () => {
  const fakeCore = renderApplicationWhileTheCoreSays({
    openBoard: { kind: "noneRunning" },
  });

  act(() => {
    fakeCore.pushOpenBoard(boardOfAda);
  });

  expect(statusBar().getByText("Ada Lovelace")).toBeDefined();
  expect(
    screen.queryByRole("heading", { name: "No board running" }),
  ).toBeNull();

  act(() => {
    fakeCore.pushOpenBoard({ kind: "noneRunning" });
  });

  expect(
    screen.getByRole("heading", { name: "No board running" }),
  ).toBeDefined();
  expect(screen.queryByRole("contentinfo")).toBeNull();
});

test("while the board is copied, the status bar says how far along the copy is", () => {
  renderApplicationWhileTheCoreSays({
    openBoard: boardOfAda,
    replicaState: { kind: "copying", stored: 42_000, total: 100_006 },
  });

  expect(statusBar().getByText("Copying 42,000 of 100,006")).toBeDefined();
});

test("when the board does not say its total, the status bar says how many are copied so far", () => {
  renderApplicationWhileTheCoreSays({
    openBoard: boardOfAda,
    replicaState: { kind: "copying", stored: 42_000, total: null },
  });

  expect(statusBar().getByText("Copying 42,000")).toBeDefined();
});

test("while the replica follows the board, the status bar says how many messages it holds", () => {
  renderApplicationWhileTheCoreSays({
    openBoard: boardOfAda,
    replicaState: { kind: "live", stored: 100_006 },
  });

  expect(statusBar().getByText("100,006 messages")).toBeDefined();
});

test("a replica of one message says so in the singular", () => {
  renderApplicationWhileTheCoreSays({
    openBoard: boardOfAda,
    replicaState: { kind: "live", stored: 1 },
  });

  expect(statusBar().getByText("1 message")).toBeDefined();
});

test("while the feed is reconnecting, the status bar says so", () => {
  renderApplicationWhileTheCoreSays({
    openBoard: boardOfAda,
    replicaState: { kind: "reconnecting", stored: 100_006 },
  });

  expect(statusBar().getByText("Reconnecting")).toBeDefined();
});

test("when the replica stops, the status bar says why", () => {
  renderApplicationWhileTheCoreSays({
    openBoard: boardOfAda,
    replicaState: {
      kind: "stopped",
      stored: 100_006,
      reason: "unwritableReplica",
    },
  });

  expect(
    statusBar().getByText("Stopped: can't write the replica"),
  ).toBeDefined();
});

test("when the board sends what the core cannot read, the status bar says so", () => {
  renderApplicationWhileTheCoreSays({
    openBoard: boardOfAda,
    replicaState: { kind: "stopped", stored: 7, reason: "unreadableBoard" },
  });

  expect(statusBar().getByText("Stopped: can't read the board")).toBeDefined();
});

test("with a board open, the sidebar lists the Flow lens, chosen", () => {
  renderApplicationWhileTheCoreSays({ openBoard: boardOfAda, channels: [] });

  const lenses = within(sidebar().getByRole("list", { name: "Lenses" }));

  expect(
    lenses
      .getByText("Flow")
      .closest("[aria-current]")
      ?.getAttribute("aria-current"),
  ).toBe("page");
});

test("the sidebar lists the board's channels as a tree", () => {
  renderApplicationWhileTheCoreSays({
    openBoard: boardOfAda,
    channels: ["fleet/api", "fleet/www", "general"],
  });

  const channels = within(sidebar().getByRole("list", { name: "Channels" }));
  const rows = channels.getAllByRole("listitem").map((row) => ({
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
  renderApplicationWhileTheCoreSays({
    openBoard: boardOfAda,
    channels: ["fleet", "fleet/www"],
  });

  const channels = within(sidebar().getByRole("list", { name: "Channels" }));

  expect(
    channels.getAllByRole("listitem").map((row) => row.textContent),
  ).toEqual(["#fleet", "#www"]);
});

test("the sidebar follows the board's channels as they appear", () => {
  const fakeCore = renderApplicationWhileTheCoreSays({
    openBoard: boardOfAda,
    channels: ["general"],
  });

  act(() => fakeCore.pushChannels(["fleet/docs", "general"]));

  const channels = within(sidebar().getByRole("list", { name: "Channels" }));
  expect(channels.getByText("#docs")).toBeDefined();
});

test("the lens says what it shows", () => {
  renderApplicationWhileTheCoreSays({ openBoard: boardOfAda, channels: [] });

  expect(screen.getByRole("heading", { name: "Flow" })).toBeDefined();
  expect(screen.getByText("Everything on the board, in order.")).toBeDefined();
});

test("with no board running, there is no sidebar", () => {
  renderApplicationWhileTheCoreSays({ openBoard: { kind: "noneRunning" } });

  expect(screen.queryByRole("complementary")).toBeNull();
});

test("channels are listed in natural order, so area2 comes before area10", () => {
  renderApplicationWhileTheCoreSays({
    openBoard: boardOfAda,
    channels: ["fleet/area1", "fleet/area10", "fleet/area2"],
  });

  const channels = within(sidebar().getByRole("list", { name: "Channels" }));

  expect(
    channels.getAllByRole("listitem").map((row) => row.textContent),
  ).toEqual(["fleet", "#area1", "#area2", "#area10"]);
});
