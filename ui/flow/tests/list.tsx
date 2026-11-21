import {
  act,
  cleanup,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import type { FlowChange, FlowRow } from "../../core/contract";
import { createFakeCore } from "../../core/fake";
import { dictionary } from "../../dictionary";
import { FlowList } from "../list";

afterEach(cleanup);

const SOMEONE_ACTING_FOR_A_PERSON = /^via /u;
const BOARD_URL = "http://127.0.0.1:4200/api/board";

function renderFlowOf(rows: FlowRow[], channel: string | null = null) {
  const fakeCore = createFakeCore();
  let latestChange: FlowChange | undefined;
  fakeCore.core.subscribeToFlow((change) => {
    latestChange = change;
  });
  fakeCore.pushFlow(rows);
  const { rerender } = render(
    <FlowList
      change={latestChange}
      channel={channel}
      boardUrl={BOARD_URL}
      readFlow={fakeCore.core.readFlow}
    />,
  );
  return {
    pushFlow: (newRows: FlowRow[]) => {
      fakeCore.pushFlow(newRows);
      act(() => {
        rerender(
          <FlowList
            change={latestChange}
            channel={channel}
            boardUrl={BOARD_URL}
            readFlow={fakeCore.core.readFlow}
          />,
        );
      });
    },
  };
}

function messageInFlow(overrides: Partial<FlowRow> & { id: string }): FlowRow {
  return {
    channel: "fleet/www",
    author: "claude",
    onBehalfOf: "ada",
    createdAt: "2026-11-21T10:00:00.000Z",
    text: "deploy failed",
    edited: false,
    deleted: false,
    firstOfItsMinute: false,
    ...overrides,
  };
}

function flowList() {
  return within(screen.getByRole("feed"));
}

test("Flow shows each message with its time, who wrote it, on whose behalf, its channel and its text", async () => {
  renderFlowOf([messageInFlow({ id: "1", text: "rolled back to d4e5f6" })]);

  const message = await flowList().findByRole("article");

  expect(within(message).getByText("claude")).toBeDefined();
  expect(
    within(message).getByText(dictionary.flow.onBehalfOf("ada")),
  ).toBeDefined();
  expect(within(message).getByText("#fleet/www")).toBeDefined();
  expect(within(message).getByText("rolled back to d4e5f6")).toBeDefined();
  expect(
    within(message).getByText(dictionary.flow.time("2026-11-21T10:00:00.000Z")),
  ).toBeDefined();
});

test("Flow lists the messages oldest first", async () => {
  renderFlowOf([
    messageInFlow({ id: "1", text: "first" }),
    messageInFlow({ id: "2", text: "second" }),
    messageInFlow({ id: "3", text: "third" }),
  ]);

  const messages = await flowList().findAllByRole("article");

  expect(
    messages.map((message) => message.getAttribute("data-message")),
  ).toEqual(["1", "2", "3"]);
});

test("a message that starts a minute shows that minute above it", async () => {
  renderFlowOf([
    messageInFlow({
      id: "1",
      firstOfItsMinute: true,
      createdAt: "2026-11-21T10:00:05.000Z",
    }),
    messageInFlow({
      id: "2",
      firstOfItsMinute: false,
      createdAt: "2026-11-21T10:00:40.000Z",
    }),
  ]);

  const [first, second] = await flowList().findAllByRole("article");
  const minute = dictionary.flow.time("2026-11-21T10:00:05.000Z");

  expect(within(first as HTMLElement).getAllByText(minute)).toHaveLength(2);
  expect(within(second as HTMLElement).getAllByText(minute)).toHaveLength(1);
});

test("a person's own message names no one it acts for", async () => {
  renderFlowOf([
    messageInFlow({ id: "1", author: "Ada Lovelace", onBehalfOf: null }),
  ]);

  const message = await flowList().findByRole("article");

  expect(within(message).queryByText(SOMEONE_ACTING_FOR_A_PERSON)).toBeNull();
});

test("a deleted message reads as deleted, and an edited one says so", async () => {
  renderFlowOf([
    messageInFlow({ id: "1", deleted: true, text: "" }),
    messageInFlow({ id: "2", edited: true, text: "fixed" }),
  ]);

  const [deleted, edited] = await flowList().findAllByRole("article");

  expect(
    within(deleted as HTMLElement).getByText(dictionary.flow.deleted),
  ).toBeDefined();
  expect(
    within(edited as HTMLElement).getByText(dictionary.flow.edited),
  ).toBeDefined();
});

test("an edit on the board redraws its message", async () => {
  const fakeCore = renderFlowOf([
    messageInFlow({ id: "1", text: "deploy failed" }),
  ]);
  await flowList().findByText("deploy failed");

  fakeCore.pushFlow([
    messageInFlow({
      id: "1",
      text: "deploy failed, rolled back",
      edited: true,
    }),
  ]);

  expect(
    await flowList().findByText("deploy failed, rolled back"),
  ).toBeDefined();
});

const SCROLLED_CONTENT = 5000;
const SCREEN = 500;

function pretendTheListIsLaidOut() {
  const scroller = screen.getByRole("feed").parentElement as HTMLElement;
  Object.defineProperty(scroller, "scrollHeight", {
    configurable: true,
    get: () => SCROLLED_CONTENT,
  });
  Object.defineProperty(scroller, "clientHeight", {
    configurable: true,
    get: () => SCREEN,
  });
  return scroller;
}

async function scrollUpTo(scroller: HTMLElement, position: number) {
  await act(async () => {
    scroller.scrollTop = position;
    scroller.dispatchEvent(new Event("scroll"));
    await new Promise((resolve) => requestAnimationFrame(resolve));
  });
}

test("scrolled up, new messages leave the reader's place alone and say how many wait below", async () => {
  const flow = renderFlowOf([
    messageInFlow({ id: "1" }),
    messageInFlow({ id: "2" }),
  ]);
  await flowList().findAllByRole("article");
  const scroller = pretendTheListIsLaidOut();
  await scrollUpTo(scroller, 0);

  flow.pushFlow([
    messageInFlow({ id: "1" }),
    messageInFlow({ id: "2" }),
    messageInFlow({ id: "3" }),
    messageInFlow({ id: "4" }),
  ]);

  expect(scroller.scrollTop).toBe(0);
  expect(
    await screen.findByRole("button", { name: dictionary.flow.newBelow(2) }),
  ).toBeDefined();
});

test("the count of new messages takes the reader down to them", async () => {
  const flow = renderFlowOf([messageInFlow({ id: "1" })]);
  await flowList().findAllByRole("article");
  const scroller = pretendTheListIsLaidOut();
  await scrollUpTo(scroller, 0);
  flow.pushFlow([messageInFlow({ id: "1" }), messageInFlow({ id: "2" })]);

  const notice = await screen.findByRole("button", {
    name: dictionary.flow.newBelow(1),
  });
  await act(async () => {
    notice.click();
    await new Promise((resolve) => requestAnimationFrame(resolve));
  });

  expect(scroller.scrollTop).toBe(SCROLLED_CONTENT);
  expect(
    screen.queryByRole("button", { name: dictionary.flow.newBelow(1) }),
  ).toBeNull();
});

test("at the bottom, new messages arrive without a count", async () => {
  const flow = renderFlowOf([messageInFlow({ id: "1" })]);
  await flowList().findAllByRole("article");

  flow.pushFlow([messageInFlow({ id: "1" }), messageInFlow({ id: "2" })]);

  await waitFor(() => {
    expect(flowList().getAllByRole("article")).toHaveLength(2);
  });
  expect(screen.queryByRole("button")).toBeNull();
});

test("a channel narrows Flow to it and to every channel under it", async () => {
  renderFlowOf(
    [
      messageInFlow({ id: "1", channel: "fleet/www" }),
      messageInFlow({ id: "2", channel: "general" }),
      messageInFlow({ id: "3", channel: "fleet" }),
      messageInFlow({ id: "4", channel: "fleetwide" }),
    ],
    "fleet",
  );

  await waitFor(() => {
    const shown = flowList()
      .getAllByRole("article")
      .map((message) => message.getAttribute("data-message"));
    expect(shown).toEqual(["1", "3"]);
  });
});

test("a board nobody has written on shows how to point an agent at it", async () => {
  renderFlowOf([]);

  expect(
    await screen.findByRole("heading", { name: dictionary.emptyBoard.title }),
  ).toBeDefined();
});
