import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import type { FlowChange, FlowRow } from "../../core/contract";
import { createFakeCore } from "../../core/fake";
import { dictionary } from "../../dictionary";
import { FlowList } from "../list";
import { ESTIMATED_ROW_HEIGHT } from "../state";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

const BOARD_URL = "http://127.0.0.1:4200/api/board";
const LINES = 200;
const LINE_AT_THE_TOP = 50;
const LANDED_ABOVE = 10;
const LANDED_BELOW = 2;
const FIRST_TALL_LINE = 40;
const TALL_LINE_HEIGHT = ESTIMATED_ROW_HEIGHT * 2;
const SCREEN = 500;
const DIGITS_IN_AN_ID = 4;
const TRANSLATION = /translateY\((?<top>.+)px\)/u;

function idOf(number: number): string {
  return String(number).padStart(DIGITS_IN_AN_ID, "0");
}

function textOfLine(number: number): string {
  const isTall = number >= FIRST_TALL_LINE && number < LINE_AT_THE_TOP;
  if (isTall) {
    return `tall line ${idOf(number)}`;
  }
  return `line ${idOf(number)}`;
}

function lineNumbered(number: number): FlowRow {
  return {
    id: idOf(number),
    channel: "fleet/www",
    author: "claude",
    onBehalfOf: "ada",
    createdAt: "2026-11-21T10:00:00.000Z",
    text: textOfLine(number),
    edited: false,
    deleted: false,
    firstOfItsMinute: false,
    fold: null,
  };
}

function linesUpTo(count: number): FlowRow[] {
  return Array.from({ length: count }, (_, number) => lineNumbered(number));
}

function olderLines(count: number): FlowRow[] {
  return Array.from({ length: count }, (_, number) => ({
    ...lineNumbered(number),
    id: `older ${idOf(number)}`,
    createdAt: "2026-11-21T09:00:00.000Z",
  }));
}

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
      countFlowBefore={fakeCore.core.countFlowBefore}
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

function pretendTheListIsLaidOut() {
  const feed = screen.getByRole("feed");
  const scroller = feed.parentElement as HTMLElement;
  Object.defineProperty(scroller, "scrollHeight", {
    configurable: true,
    get: () => Number.parseFloat(feed.style.height),
  });
  Object.defineProperty(scroller, "clientHeight", {
    configurable: true,
    get: () => SCREEN,
  });
  return scroller;
}

function pretendTallLinesAreTwiceAsTall() {
  vi.spyOn(HTMLElement.prototype, "offsetHeight", "get").mockImplementation(
    function heightOf(this: HTMLElement) {
      if (this.dataset.loaded === undefined) {
        return 0;
      }
      if (this.textContent?.includes("tall") === true) {
        return TALL_LINE_HEIGHT;
      }
      return ESTIMATED_ROW_HEIGHT;
    },
  );
}

async function scrollTo(scroller: HTMLElement, position: number) {
  await act(async () => {
    scroller.scrollTop = position;
    scroller.dispatchEvent(new Event("scroll"));
    await new Promise((resolve) => requestAnimationFrame(resolve));
  });
}

async function scrollUpFromTheBottomTo(scroller: HTMLElement, line: number) {
  await scrollTo(scroller, scroller.scrollHeight - SCREEN);
  await screen.findByText(`line ${idOf(LINES - 1)}`);
  await scrollTo(scroller, line * ESTIMATED_ROW_HEIGHT);
  await screen.findByText(`line ${idOf(line)}`);
}

function wrapperOf(line: number): HTMLElement {
  const article = document.querySelector(`[data-message="${idOf(line)}"]`);
  const wrapper = article?.closest<HTMLElement>("[data-index]");
  if (wrapper === null || wrapper === undefined) {
    throw new Error(`line ${idOf(line)} is not on screen`);
  }
  return wrapper;
}

function heightOnScreenOf(line: number, scroller: HTMLElement): number {
  const top = Number(
    TRANSLATION.exec(wrapperOf(line).style.transform)?.groups?.top,
  );
  return top - scroller.scrollTop;
}

test("scrolled up, the line at the top of the screen stays there while older messages land above, and only what lands below is counted", async () => {
  const flow = renderFlowOf(linesUpTo(LINES));
  await screen.findAllByRole("article");
  const scroller = pretendTheListIsLaidOut();
  await scrollUpFromTheBottomTo(scroller, LINE_AT_THE_TOP);

  flow.pushFlow([
    ...olderLines(LANDED_ABOVE),
    ...linesUpTo(LINES + LANDED_BELOW),
  ]);

  const whereItSitsNow =
    (LINE_AT_THE_TOP + LANDED_ABOVE) * ESTIMATED_ROW_HEIGHT;
  await waitFor(() => {
    expect(scroller.scrollTop).toBe(whereItSitsNow);
  });
  expect(wrapperOf(LINE_AT_THE_TOP).dataset.index).toBe(
    String(LINE_AT_THE_TOP + LANDED_ABOVE),
  );
  expect(heightOnScreenOf(LINE_AT_THE_TOP, scroller)).toBe(0);
  expect(
    await screen.findByRole("button", {
      name: dictionary.flow.newBelow(LANDED_BELOW),
    }),
  ).toBeDefined();
});

test("a change that comes while the line on screen is being found again holds it from where it was found", async () => {
  const flow = renderFlowOf(linesUpTo(LINES));
  await screen.findAllByRole("article");
  const scroller = pretendTheListIsLaidOut();
  await scrollUpFromTheBottomTo(scroller, LINE_AT_THE_TOP);

  flow.pushFlow([...olderLines(LANDED_ABOVE), ...linesUpTo(LINES)]);
  flow.pushFlow([...olderLines(LANDED_ABOVE * 2), ...linesUpTo(LINES)]);

  const landedInAll = LANDED_ABOVE * 2;
  await waitFor(() => {
    expect(scroller.scrollTop).toBe(
      (LINE_AT_THE_TOP + landedInAll) * ESTIMATED_ROW_HEIGHT,
    );
  });
  expect(heightOnScreenOf(LINE_AT_THE_TOP, scroller)).toBe(0);
});

test("lines above that measure otherwise after the change leave the line on screen where it was", async () => {
  pretendTallLinesAreTwiceAsTall();
  const flow = renderFlowOf(linesUpTo(LINES));
  await screen.findAllByRole("article");
  const scroller = pretendTheListIsLaidOut();
  await scrollUpFromTheBottomTo(scroller, LINE_AT_THE_TOP);
  expect(heightOnScreenOf(LINE_AT_THE_TOP, scroller)).toBe(0);

  flow.pushFlow([...olderLines(LANDED_ABOVE), ...linesUpTo(LINES)]);

  await waitFor(() => {
    expect(wrapperOf(LINE_AT_THE_TOP).dataset.index).toBe(
      String(LINE_AT_THE_TOP + LANDED_ABOVE),
    );
  });
  expect(heightOnScreenOf(LINE_AT_THE_TOP, scroller)).toBe(0);
});
