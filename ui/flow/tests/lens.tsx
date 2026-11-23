import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import type {
  FlowChange,
  FlowRow,
  Thread,
  ThreadMessage,
} from "../../core/contract";
import { createFakeCore } from "../../core/fake";
import { dictionary } from "../../dictionary";
import { createThreadOpenings } from "../../thread/openings";
import { FlowLens } from "../lens";
import { ESTIMATED_ROW_HEIGHT } from "../state";

afterEach(cleanup);

const SCROLLED_CONTENT = 5000;
const SCREEN = 500;
const LINES_ABOVE_THE_THIRD = 2;

function messageInFlow(id: string, text: string): FlowRow {
  return {
    id,
    channel: `fleet/${id}`,
    author: "claude",
    onBehalfOf: "ada",
    createdAt: "2026-11-21T10:00:00.000Z",
    text,
    edited: false,
    deleted: false,
    firstOfItsMinute: false,
    fold: null,
  };
}

function threadOfItsOwn(row: FlowRow): Thread {
  const root: ThreadMessage = {
    id: row.id,
    author: row.author,
    onBehalfOf: row.onBehalfOf,
    createdAt: row.createdAt,
    text: row.text,
    edited: row.edited,
    deleted: row.deleted,
  };
  return { id: `thread-${row.id}`, channel: row.channel, messages: [root] };
}

const firstLine = messageInFlow("first", "deploy www");
const secondLine = messageInFlow("second", "deploy api");
const thirdLine = messageInFlow("third", "deploy docs");

function renderFlowLensOf(rows: FlowRow[]) {
  const fakeCore = createFakeCore();
  let latestChange: FlowChange | undefined;
  fakeCore.core.subscribeToFlow((change) => {
    latestChange = change;
  });
  fakeCore.pushFlow(rows);
  fakeCore.pushThreads(rows.map(threadOfItsOwn));
  const threadOpenings = createThreadOpenings();
  render(
    <FlowLens
      change={latestChange}
      channel={null}
      boardUrl="http://127.0.0.1:4200/api/board"
      onWiden={vi.fn()}
      readFlow={fakeCore.core.readFlow}
      countFlowBefore={fakeCore.core.countFlowBefore}
      countFlowAfter={fakeCore.core.countFlowAfter}
      readThread={fakeCore.core.readThread}
      openLink={fakeCore.core.openLink}
      threadOpenings={threadOpenings}
    />,
  );
  return { threadOpenings };
}

function flowLines() {
  return within(screen.getByRole("main")).findAllByRole("article");
}

async function lineOf(row: FlowRow) {
  const lines = await flowLines();
  const line = lines.find(
    (candidate) => candidate.getAttribute("data-message") === row.id,
  );
  if (line === undefined) {
    throw new Error(`Flow draws no line for ${row.id}`);
  }
  return line;
}

async function selectedLines() {
  const lines = await flowLines();
  return lines
    .filter((line) => line.getAttribute("aria-current") === "true")
    .map((line) => line.getAttribute("data-message"));
}

function paneFor(row: FlowRow) {
  return screen.findByRole("complementary", {
    name: dictionary.thread.inChannel(row.channel),
  });
}

function press(key: string) {
  act(() => {
    fireEvent.keyDown(document.body, { key });
  });
}

async function pretendTheListIsLaidOutAtTheBottom() {
  const scroller = screen.getByRole("feed").parentElement as HTMLElement;
  Object.defineProperty(scroller, "scrollHeight", {
    configurable: true,
    get: () => SCROLLED_CONTENT,
  });
  Object.defineProperty(scroller, "clientHeight", {
    configurable: true,
    get: () => SCREEN,
  });
  await act(async () => {
    scroller.scrollTop = SCROLLED_CONTENT;
    scroller.dispatchEvent(new Event("scroll"));
    await new Promise((resolve) => requestAnimationFrame(resolve));
  });
  return scroller;
}

async function selectTheLineOf(row: FlowRow) {
  const line = await lineOf(row);
  act(() => {
    line.click();
  });
  await paneFor(row);
}

test("until a line is selected, no pane is open", async () => {
  renderFlowLensOf([firstLine, secondLine]);

  await flowLines();

  expect(screen.queryByRole("complementary")).toBeNull();
});

test("selecting a line opens its thread in the pane and marks the line", async () => {
  renderFlowLensOf([firstLine, secondLine]);

  await selectTheLineOf(secondLine);

  const pane = await paneFor(secondLine);
  expect(await within(pane).findByText("deploy api")).toBeDefined();
  expect(await selectedLines()).toEqual(["second"]);
});

test("a thread opened from elsewhere opens in the pane and marks its line", async () => {
  const lens = renderFlowLensOf([firstLine, secondLine]);
  await flowLines();

  act(() => {
    lens.threadOpenings.open("second");
  });

  const pane = await paneFor(secondLine);
  expect(await within(pane).findByText("deploy api")).toBeDefined();
  expect(await selectedLines()).toEqual(["second"]);
});

test("Esc closes the pane and lets go of the line", async () => {
  renderFlowLensOf([firstLine, secondLine]);
  await selectTheLineOf(firstLine);

  press("Escape");

  expect(screen.queryByRole("complementary")).toBeNull();
  expect(await selectedLines()).toEqual([]);
});

test("the arrow keys move the selection through Flow, and the pane follows", async () => {
  renderFlowLensOf([firstLine, secondLine, thirdLine]);
  await selectTheLineOf(firstLine);

  press("ArrowDown");
  press("ArrowDown");

  expect(await paneFor(thirdLine)).toBeDefined();
  expect(await selectedLines()).toEqual(["third"]);

  press("ArrowUp");

  expect(await paneFor(secondLine)).toBeDefined();
  expect(await selectedLines()).toEqual(["second"]);
});

test("the selection stops at the oldest and the newest lines", async () => {
  renderFlowLensOf([firstLine, secondLine]);
  await selectTheLineOf(firstLine);

  press("ArrowUp");

  expect(await selectedLines()).toEqual(["first"]);

  press("ArrowDown");
  press("ArrowDown");

  await waitFor(async () => {
    expect(await selectedLines()).toEqual(["second"]);
  });
});

test("with nothing selected, an arrow selects the line at the bottom of the screen", async () => {
  renderFlowLensOf([firstLine, secondLine, thirdLine]);
  await flowLines();
  await pretendTheListIsLaidOutAtTheBottom();

  press("ArrowUp");

  expect(await paneFor(thirdLine)).toBeDefined();
  expect(await selectedLines()).toEqual(["third"]);
});

test("an arrow scrolls Flow to the line it selects, away from the bottom", async () => {
  renderFlowLensOf([firstLine, secondLine, thirdLine]);
  await flowLines();
  const scroller = await pretendTheListIsLaidOutAtTheBottom();

  press("ArrowUp");
  await paneFor(thirdLine);

  expect(scroller.scrollTop).toBe(LINES_ABOVE_THE_THIRD * ESTIMATED_ROW_HEIGHT);
});

test("while another control has the keyboard, the arrows leave the selection alone", async () => {
  renderFlowLensOf([firstLine, secondLine]);
  await selectTheLineOf(firstLine);
  const close = within(await paneFor(firstLine)).getByRole("button", {
    name: dictionary.thread.close,
  });
  close.focus();

  act(() => {
    fireEvent.keyDown(close, { key: "ArrowDown" });
  });

  expect(await selectedLines()).toEqual(["first"]);
});
