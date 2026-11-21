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
import type { FlowChange, Thread, ThreadMessage } from "../../core/contract";
import { createFakeCore } from "../../core/fake";
import { dictionary } from "../../dictionary";
import { ThreadPane } from "../pane";

afterEach(cleanup);

const SOMEONE_ACTING_FOR_A_PERSON = /^via /u;
const THREAD_HEIGHT = 4000;
const PANE_HEIGHT = 600;
const PLACE_IN_THE_MIDDLE = 1200;

function messageInThread(
  overrides: Partial<ThreadMessage> & { id: string },
): ThreadMessage {
  return {
    author: "claude",
    onBehalfOf: "ada",
    createdAt: "2026-11-21T10:00:00.000Z",
    text: "deploy www",
    edited: false,
    deleted: false,
    ...overrides,
  };
}

function threadOf(messages: ThreadMessage[]): Thread {
  return { id: "thread-root", channel: "fleet/www", messages };
}

function renderPaneOf(messageId: string, threads: Thread[]) {
  const fakeCore = createFakeCore();
  let latestChange: FlowChange | undefined;
  const close = vi.fn();
  fakeCore.core.subscribeToFlow((change) => {
    latestChange = change;
  });
  fakeCore.pushThreads(threads);
  const pane = () => (
    <ThreadPane
      messageId={messageId}
      change={latestChange}
      readThread={fakeCore.core.readThread}
      onClose={close}
    />
  );
  const { rerender } = render(pane());
  return {
    pushThreads: (newThreads: Thread[]) => {
      fakeCore.pushThreads(newThreads);
      act(() => {
        rerender(pane());
      });
    },
    closings: () => close.mock.calls.length,
  };
}

function threadPane() {
  return within(screen.getByRole("complementary"));
}

function messageIds(messages: HTMLElement[]) {
  return messages.map((message) => message.getAttribute("data-message"));
}

function pretendThePaneIsLaidOut() {
  const body = threadPane().getByRole("feed");
  Object.defineProperty(body, "scrollHeight", {
    configurable: true,
    get: () => THREAD_HEIGHT,
  });
  Object.defineProperty(body, "clientHeight", {
    configurable: true,
    get: () => PANE_HEIGHT,
  });
  return body;
}

test("the pane shows the thread's root and every reply, oldest first", async () => {
  renderPaneOf("reply", [
    threadOf([
      messageInThread({ id: "root", text: "deploy www" }),
      messageInThread({ id: "reply", text: "deploy failed" }),
      messageInThread({ id: "later", text: "rolled back" }),
    ]),
  ]);

  const messages = await threadPane().findAllByRole("article");

  expect(messageIds(messages)).toEqual(["root", "reply", "later"]);
});

test("each message says who wrote it, on whose behalf, and when", async () => {
  renderPaneOf("root", [
    threadOf([
      messageInThread({ id: "root", createdAt: "2026-11-21T10:07:00.000Z" }),
    ]),
  ]);

  const message = await threadPane().findByRole("article");

  expect(within(message).getByText("claude")).toBeDefined();
  expect(
    within(message).getByText(dictionary.thread.onBehalfOf("ada")),
  ).toBeDefined();
  expect(
    within(message).getByText(
      dictionary.thread.time("2026-11-21T10:07:00.000Z"),
    ),
  ).toBeDefined();
});

test("a person's own message names no one it acts for", async () => {
  renderPaneOf("root", [
    threadOf([
      messageInThread({ id: "root", author: "Ada Lovelace", onBehalfOf: null }),
    ]),
  ]);

  const message = await threadPane().findByRole("article");

  expect(within(message).queryByText(SOMEONE_ACTING_FOR_A_PERSON)).toBeNull();
});

test("a message's text reads as plain text, as it was written", async () => {
  const written = "## Report\n\n  - **built** <b>www</b>";
  renderPaneOf("root", [
    threadOf([messageInThread({ id: "root", text: written })]),
  ]);

  const message = await threadPane().findByRole("article");

  expect(
    within(message).getByText(written, { normalizer: String }),
  ).toBeDefined();
});

test("a deleted message reads as deleted, and an edited one says so", async () => {
  renderPaneOf("root", [
    threadOf([
      messageInThread({ id: "root", deleted: true, text: "" }),
      messageInThread({ id: "reply", edited: true, text: "fixed" }),
    ]),
  ]);

  const [deleted, edited] = await threadPane().findAllByRole("article");

  expect(
    within(deleted as HTMLElement).getByText(dictionary.thread.deleted),
  ).toBeDefined();
  expect(
    within(edited as HTMLElement).getByText(dictionary.thread.edited),
  ).toBeDefined();
});

test("the pane is named after the thread's channel", async () => {
  renderPaneOf("root", [threadOf([messageInThread({ id: "root" })])]);

  expect(
    await screen.findByRole("complementary", {
      name: dictionary.thread.inChannel("fleet/www"),
    }),
  ).toBeDefined();
});

test("a reply that arrives joins the open thread", async () => {
  const pane = renderPaneOf("root", [
    threadOf([messageInThread({ id: "root" })]),
  ]);
  await threadPane().findByRole("article");

  pane.pushThreads([
    threadOf([
      messageInThread({ id: "root" }),
      messageInThread({ id: "reply", text: "deploy failed" }),
    ]),
  ]);

  expect(await threadPane().findByText("deploy failed")).toBeDefined();
});

test("an edit on the board redraws its message in the pane", async () => {
  const pane = renderPaneOf("root", [
    threadOf([messageInThread({ id: "root", text: "deploy failed" })]),
  ]);
  await threadPane().findByText("deploy failed");

  pane.pushThreads([
    threadOf([
      messageInThread({
        id: "root",
        text: "deploy failed, rolled back",
        edited: true,
      }),
    ]),
  ]);

  expect(
    await threadPane().findByText("deploy failed, rolled back"),
  ).toBeDefined();
});

test("at the end of the thread, a reply that arrives brings the pane down to it", async () => {
  const pane = renderPaneOf("root", [
    threadOf([messageInThread({ id: "root" })]),
  ]);
  await threadPane().findByRole("article");
  const body = pretendThePaneIsLaidOut();
  body.scrollTop = THREAD_HEIGHT - PANE_HEIGHT;

  pane.pushThreads([
    threadOf([
      messageInThread({ id: "root" }),
      messageInThread({ id: "reply", text: "deploy failed" }),
    ]),
  ]);
  await threadPane().findByText("deploy failed");

  expect(body.scrollTop).toBe(THREAD_HEIGHT);
});

test("further up the thread, a reply that arrives leaves the reader's place alone", async () => {
  const pane = renderPaneOf("root", [
    threadOf([messageInThread({ id: "root" })]),
  ]);
  await threadPane().findByRole("article");
  const body = pretendThePaneIsLaidOut();
  body.scrollTop = PLACE_IN_THE_MIDDLE;

  pane.pushThreads([
    threadOf([
      messageInThread({ id: "root" }),
      messageInThread({ id: "reply", text: "deploy failed" }),
    ]),
  ]);
  await threadPane().findByText("deploy failed");

  expect(body.scrollTop).toBe(PLACE_IN_THE_MIDDLE);
});

test("Esc closes the pane", async () => {
  const pane = renderPaneOf("root", [
    threadOf([messageInThread({ id: "root" })]),
  ]);
  await threadPane().findByRole("article");

  fireEvent.keyDown(document.body, { key: "Escape" });

  expect(pane.closings()).toBe(1);
});

test("the pane's Close button closes it", () => {
  const pane = renderPaneOf("root", [
    threadOf([messageInThread({ id: "root" })]),
  ]);

  threadPane().getByRole("button", { name: dictionary.thread.close }).click();

  expect(pane.closings()).toBe(1);
});

test("a message the replica no longer holds closes the pane", async () => {
  const pane = renderPaneOf("gone", [
    threadOf([messageInThread({ id: "root" })]),
  ]);

  await waitFor(() => {
    expect(pane.closings()).toBe(1);
  });
});
