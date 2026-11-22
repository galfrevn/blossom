import {
  act,
  cleanup,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import type { FlowChange, FlowRow } from "../../core/contract";
import { createFakeCore } from "../../core/fake";
import { dictionary } from "../../dictionary";
import { FlowList } from "../list";

afterEach(cleanup);

const BOARD_URL = "http://127.0.0.1:4200/api/board";

function renderFlowOf(rows: FlowRow[]) {
  const fakeCore = createFakeCore();
  let latestChange: FlowChange | undefined;
  fakeCore.core.subscribeToFlow((change) => {
    latestChange = change;
  });
  fakeCore.pushFlow(rows);
  const selectNothing = vi.fn();
  const list = () => (
    <FlowList
      change={latestChange}
      channel={null}
      boardUrl={BOARD_URL}
      readFlow={fakeCore.core.readFlow}
      countFlowAfter={fakeCore.core.countFlowAfter}
      selectedMessage={undefined}
      onSelect={selectNothing}
    />
  );
  const { rerender } = render(list());
  return {
    pushFlow: (newRows: FlowRow[]) => {
      fakeCore.pushFlow(newRows);
      act(() => {
        rerender(list());
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
    fold: null,
    ...overrides,
  };
}

function flowList() {
  return within(screen.getByRole("feed"));
}

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

const THE_BOTTOM = SCROLLED_CONTENT - SCREEN;

async function scrollTo(scroller: HTMLElement, position: number) {
  await act(async () => {
    scroller.scrollTop = position;
    scroller.dispatchEvent(new Event("scroll"));
    await new Promise((resolve) => requestAnimationFrame(resolve));
  });
}

async function scrollUpTo(scroller: HTMLElement, position: number) {
  await scrollTo(scroller, THE_BOTTOM);
  await scrollTo(scroller, position);
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

const AN_HOUR_EARLIER = "2026-11-21T09:00:00.000Z";

test("scrolled up, older messages that land above count none of them among the new ones below", async () => {
  const flow = renderFlowOf([
    messageInFlow({ id: "1" }),
    messageInFlow({ id: "2" }),
  ]);
  await flowList().findAllByRole("article");
  const scroller = pretendTheListIsLaidOut();
  await scrollUpTo(scroller, 0);

  flow.pushFlow([
    messageInFlow({ id: "older", createdAt: AN_HOUR_EARLIER }),
    messageInFlow({ id: "oldest", createdAt: AN_HOUR_EARLIER }),
    messageInFlow({ id: "1" }),
    messageInFlow({ id: "2" }),
    messageInFlow({ id: "3" }),
  ]);

  expect(
    await screen.findByRole("button", { name: dictionary.flow.newBelow(1) }),
  ).toBeDefined();
});

test("at the bottom, older messages that land above leave the newest in view", async () => {
  const flow = renderFlowOf([
    messageInFlow({ id: "1" }),
    messageInFlow({ id: "2" }),
  ]);
  await flowList().findAllByRole("article");
  const scroller = pretendTheListIsLaidOut();

  flow.pushFlow([
    messageInFlow({ id: "older", createdAt: AN_HOUR_EARLIER }),
    messageInFlow({ id: "oldest", createdAt: AN_HOUR_EARLIER }),
    messageInFlow({ id: "1" }),
    messageInFlow({ id: "2", text: "the newest, still in view" }),
  ]);

  expect(
    await flowList().findByText("the newest, still in view"),
  ).toBeDefined();
  expect(scroller.scrollTop).toBe(SCROLLED_CONTENT);
  expect(screen.queryByRole("button")).toBeNull();
});

const GROWN_CONTENT = 5300;

function pretendItScrollsNoFurtherThanTheEnd(scroller: HTMLElement) {
  let position = 0;
  Object.defineProperty(scroller, "scrollTop", {
    configurable: true,
    get: () => position,
    set: (wanted: number) => {
      position = Math.min(
        wanted,
        scroller.scrollHeight - scroller.clientHeight,
      );
    },
  });
}

function pretendTheContentGrew(scroller: HTMLElement) {
  Object.defineProperty(scroller, "scrollHeight", {
    configurable: true,
    get: () => GROWN_CONTENT,
  });
}

test("at the bottom, lines that grow taller where the list cannot measure them leave it at the bottom", async () => {
  const flow = renderFlowOf([messageInFlow({ id: "1" })]);
  await flowList().findAllByRole("article");
  const scroller = pretendTheListIsLaidOut();
  pretendItScrollsNoFurtherThanTheEnd(scroller);
  await scrollTo(scroller, THE_BOTTOM);

  pretendTheContentGrew(scroller);
  await scrollTo(scroller, THE_BOTTOM);
  flow.pushFlow([
    messageInFlow({ id: "1" }),
    messageInFlow({ id: "2", text: "arrived while the lines grew" }),
  ]);

  expect(
    await flowList().findByText("arrived while the lines grew"),
  ).toBeDefined();
  expect(scroller.scrollTop).toBe(GROWN_CONTENT - SCREEN);
  expect(screen.queryByRole("button")).toBeNull();
});
