import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { Application } from "../application";
import { createClock } from "../core/clock";
import type {
  FlowRow,
  OpenBoard,
  ReplicaState,
  SearchResults,
  Thread,
} from "../core/contract";
import { createFakeCore, type FakeCore } from "../core/fake";
import { createStore } from "../core/store";
import { dictionary } from "../dictionary";

afterEach(cleanup);

const ONE_SECOND = 1000;

const boardOfAda: OpenBoard = {
  kind: "open",
  url: "http://127.0.0.1:4100/api/board",
  person: { handle: "ada", name: "Ada Lovelace" },
};

function renderApplicationWhileTheCoreSays(said: {
  openBoard?: OpenBoard;
  replicaState?: ReplicaState;
  channels?: string[];
  flow?: FlowRow[];
  threads?: Thread[];
}) {
  const fakeCore = createFakeCore();
  const openBoardStore = createStore(fakeCore.core.subscribeToOpenBoard);
  const replicaStore = createStore(fakeCore.core.subscribeToReplica);
  const channelsStore = createStore(fakeCore.core.subscribeToChannels);
  const flowStore = createStore(fakeCore.core.subscribeToFlow);
  const tasksStore = createStore(fakeCore.core.subscribeToTasks);
  if (said.openBoard !== undefined) {
    fakeCore.pushOpenBoard(said.openBoard);
  }
  if (said.replicaState !== undefined) {
    fakeCore.pushReplicaState(said.replicaState);
  }
  if (said.channels !== undefined) {
    fakeCore.pushChannels(said.channels);
  }
  if (said.flow !== undefined) {
    fakeCore.pushFlow(said.flow);
  }
  if (said.threads !== undefined) {
    fakeCore.pushThreads(said.threads);
  }
  render(
    <Application
      openBoardStore={openBoardStore}
      replicaStore={replicaStore}
      channelsStore={channelsStore}
      flowStore={flowStore}
      tasksStore={tasksStore}
      clock={createClock(ONE_SECOND)}
      readFlow={fakeCore.core.readFlow}
      countFlowBefore={fakeCore.core.countFlowBefore}
      countFlowAfter={fakeCore.core.countFlowAfter}
      readThread={fakeCore.core.readThread}
      openLink={fakeCore.core.openLink}
      readTasks={fakeCore.core.readTasks}
      readTaskCounts={fakeCore.core.readTaskCounts}
      search={fakeCore.core.search}
    />,
  );
  return fakeCore;
}

const rootOfTheDeploy = {
  id: "root",
  author: "Ada Lovelace",
  onBehalfOf: null,
  createdAt: "2026-11-21T10:00:00.000Z",
  text: "deploy www",
  edited: false,
  deleted: false,
};

const threadOfTheDeploy: Thread = {
  id: "thread-root",
  channel: "fleet/www",
  messages: [
    rootOfTheDeploy,
    {
      id: "reply",
      author: "claude",
      onBehalfOf: "ada",
      createdAt: "2026-11-21T10:01:00.000Z",
      text: "deploy failed",
      edited: false,
      deleted: false,
    },
  ],
};

function statusBar() {
  return within(screen.getByRole("contentinfo"));
}

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

test("while the replica follows the board, the status bar says how many messages it holds", () => {
  renderApplicationWhileTheCoreSays({
    openBoard: boardOfAda,
    replicaState: { kind: "live", stored: 100_006 },
  });

  expect(statusBar().getByText("100,006 messages")).toBeDefined();
});

test("the lens says what it shows", () => {
  renderApplicationWhileTheCoreSays({ openBoard: boardOfAda, channels: [] });

  expect(screen.getByRole("heading", { name: "Flow" })).toBeDefined();
  expect(screen.getByText(dictionary.lenses.flow.description)).toBeDefined();
});

test("with no board running, there is neither sidebar nor finder", () => {
  renderApplicationWhileTheCoreSays({ openBoard: { kind: "noneRunning" } });

  expect(screen.queryByRole("complementary")).toBeNull();
  expect(
    screen.queryByRole("button", { name: dictionary.finder.box }),
  ).toBeNull();
});

test("selecting a line in Flow opens its thread in the pane, beside the lens", async () => {
  renderApplicationWhileTheCoreSays({
    openBoard: boardOfAda,
    channels: ["fleet/www"],
    flow: [
      {
        id: "reply",
        channel: "fleet/www",
        author: "claude",
        onBehalfOf: "ada",
        createdAt: "2026-11-21T10:01:00.000Z",
        text: "deploy failed",
        edited: false,
        deleted: false,
        firstOfItsMinute: true,
        fold: null,
      },
    ],
    threads: [threadOfTheDeploy],
  });
  const line = await within(screen.getByRole("main")).findByRole("article");

  act(() => {
    line.click();
  });

  const pane = await screen.findByRole("complementary", {
    name: dictionary.thread.inChannel("fleet/www"),
  });
  expect(await within(pane).findByText("deploy www")).toBeDefined();
  expect(within(pane).getByText("deploy failed")).toBeDefined();
});

test("choosing a lens in the sidebar shows it, still narrowed to the chosen channel", () => {
  renderApplicationWhileTheCoreSays({
    openBoard: boardOfAda,
    channels: ["fleet/www"],
  });

  act(() => {
    screen.getByRole("button", { name: "#www" }).click();
  });
  act(() => {
    screen.getByRole("button", { name: dictionary.lenses.tasks.title }).click();
  });

  expect(
    screen.getByRole("heading", { name: dictionary.lenses.tasks.title }),
  ).toBeDefined();
  expect(
    screen.getByText(dictionary.lenses.narrowedTo("fleet/www")),
  ).toBeDefined();
});

async function openInTheFinder(fakeCore: FakeCore, found: SearchResults) {
  fakeCore.answerSearch("www", found);
  act(() => {
    screen.getByRole("button", { name: dictionary.finder.box }).click();
  });
  const box = screen.getByRole("combobox");
  fireEvent.change(box, { target: { value: "www" } });
  await screen.findByRole("option");
  act(() => {
    fireEvent.keyDown(box, { key: "Enter" });
  });
}

test("a message found in the finder opens its thread in the pane, beside the lens", async () => {
  const fakeCore = renderApplicationWhileTheCoreSays({
    openBoard: boardOfAda,
    threads: [threadOfTheDeploy],
  });
  const { id, author, createdAt } = rootOfTheDeploy;
  const message = { id, author, createdAt, channel: "fleet/www", snippet: [] };

  await openInTheFinder(fakeCore, {
    messages: [message],
    tasks: [],
    channels: [],
    agents: [],
  });

  const pane = await screen.findByRole("complementary", {
    name: dictionary.thread.inChannel("fleet/www"),
  });
  expect(await within(pane).findByText("deploy www")).toBeDefined();
  expect(screen.queryByRole("dialog")).toBeNull();
});

test("a channel found in the finder narrows the lens to it", async () => {
  const fakeCore = renderApplicationWhileTheCoreSays({ openBoard: boardOfAda });
  const channel = { path: "fleet/www", markedPath: [] };

  await openInTheFinder(fakeCore, {
    messages: [],
    tasks: [],
    channels: [channel],
    agents: [],
  });

  expect(
    screen.getByText(dictionary.lenses.narrowedTo("fleet/www")),
  ).toBeDefined();
});
