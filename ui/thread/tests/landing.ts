import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import type { Thread, ThreadMessage } from "../../core/contract";
import { MARK_LASTS, useLandingOnTheMessageAskedFor } from "../landing";

const REPLIES = 200;

let scrolledIntoView: HTMLElement[] = [];

beforeEach(() => {
  scrolledIntoView = [];
  vi.spyOn(HTMLElement.prototype, "scrollIntoView").mockImplementation(
    function (this: HTMLElement) {
      scrolledIntoView.push(this);
    },
  );
});

afterEach(() => {
  cleanup();
  vi.useRealTimers();
  vi.restoreAllMocks();
  document.body.replaceChildren();
});

function message(id: string): ThreadMessage {
  return {
    id,
    author: "claude",
    onBehalfOf: null,
    createdAt: "2026-11-21T10:00:00.000Z",
    text: id,
    mentions: [],
    edited: false,
    deleted: false,
  };
}

const ids = ["root", ...Array.from({ length: REPLIES }, (_, at) => `r${at}`)];

function threadOfEveryReply(): Thread {
  return { id: "root", channel: "fleet/www", messages: ids.map(message) };
}

function bodyHoldingEveryReply(): HTMLElement {
  const body = document.createElement("div");
  for (const id of ids) {
    const article = document.createElement("article");
    article.dataset.message = id;
    body.append(article);
  }
  document.body.append(body);
  return body;
}

function landOn(messageId: string, thread: Thread | undefined) {
  const body = { current: bodyHoldingEveryReply() };
  return renderHook(
    (props: { messageId: string; thread: Thread | undefined }) =>
      useLandingOnTheMessageAskedFor(body, props.thread, props.messageId),
    { initialProps: { messageId, thread } },
  );
}

function messagesScrolledIntoView(): (string | undefined)[] {
  return scrolledIntoView.map((element) => element.dataset.message);
}

test("a reply asked for is brought into view and marked", () => {
  const landing = landOn("r180", threadOfEveryReply());

  expect(messagesScrolledIntoView()).toEqual(["r180"]);
  expect(landing.result.current).toBe("r180");
});

test("the mark fades after a moment", () => {
  vi.useFakeTimers();
  const landing = landOn("r180", threadOfEveryReply());

  act(() => {
    vi.advanceTimersByTime(MARK_LASTS);
  });

  expect(landing.result.current).toBeUndefined();
});

test("the root asked for leaves the pane at the top, unmarked", () => {
  const landing = landOn("root", threadOfEveryReply());

  expect(messagesScrolledIntoView()).toEqual([]);
  expect(landing.result.current).toBeUndefined();
});

test("until its thread is read, nothing is brought into view", () => {
  const landing = landOn("r180", undefined);

  expect(messagesScrolledIntoView()).toEqual([]);

  landing.rerender({ messageId: "r180", thread: threadOfEveryReply() });

  expect(messagesScrolledIntoView()).toEqual(["r180"]);
});

test("the thread read again leaves the pane where the reader has it", () => {
  const landing = landOn("r180", threadOfEveryReply());

  landing.rerender({ messageId: "r180", thread: threadOfEveryReply() });

  expect(messagesScrolledIntoView()).toEqual(["r180"]);
});

test("another reply of the same thread asked for is brought into view", () => {
  const landing = landOn("r180", threadOfEveryReply());

  landing.rerender({ messageId: "r20", thread: threadOfEveryReply() });

  expect(messagesScrolledIntoView()).toEqual(["r180", "r20"]);
  expect(landing.result.current).toBe("r20");
});

test("a message the thread read does not hold waits for the thread that holds it", () => {
  const landing = landOn("elsewhere", threadOfEveryReply());

  expect(messagesScrolledIntoView()).toEqual([]);
  expect(landing.result.current).toBeUndefined();
});
