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
    mentions: [],
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
  let openMessageId = messageId;
  const close = vi.fn();
  fakeCore.core.subscribeToFlow((change) => {
    latestChange = change;
  });
  fakeCore.pushThreads(threads);
  const pane = () => (
    <ThreadPane
      messageId={openMessageId}
      change={latestChange}
      readThread={fakeCore.core.readThread}
      openLink={fakeCore.core.openLink}
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
    open: (otherMessageId: string) => {
      openMessageId = otherMessageId;
      act(() => {
        rerender(pane());
      });
    },
    closings: () => close.mock.calls.length,
    linksOpened: fakeCore.linksOpened,
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

test("a click on a link opens it in the browser, and the window stays where it is", async () => {
  const pane = renderPaneOf("root", [
    threadOf([
      messageInThread({
        id: "root",
        text: "Preview at [www](https://www-git-fix.vercel.app)",
      }),
    ]),
  ]);
  const link = await threadPane().findByRole("link", { name: "www" });

  const theWindowWouldFollowTheLink = fireEvent.click(link);

  expect(theWindowWouldFollowTheLink).toBe(false);
  expect(pane.linksOpened()).toEqual(["https://www-git-fix.vercel.app/"]);
});

test("a click on what a link holds opens the link", async () => {
  const pane = renderPaneOf("root", [
    threadOf([
      messageInThread({
        id: "root",
        text: "[`vb serve` docs](https://vercel.com/docs)",
      }),
    ]),
  ]);
  const link = await threadPane().findByRole("link");

  fireEvent.click(within(link).getByText("vb serve"));

  expect(pane.linksOpened()).toEqual(["https://vercel.com/docs"]);
});

test("the reply the pane was opened at is marked", async () => {
  renderPaneOf("reply", [
    threadOf([
      messageInThread({ id: "root" }),
      messageInThread({ id: "reply" }),
    ]),
  ]);
  const messages = await threadPane().findAllByRole("article");

  await waitFor(() => {
    expect(messages.map((message) => message.dataset.landed)).toEqual([
      "false",
      "true",
    ]);
  });
});

test("after the pane moves to another thread, its links still open", async () => {
  const pane = renderPaneOf("first", [
    { ...threadOf([messageInThread({ id: "first" })]), id: "first" },
    {
      ...threadOf([
        messageInThread({
          id: "second",
          text: "logs at https://vercel.com/logs",
        }),
      ]),
      id: "second",
    },
  ]);
  await threadPane().findByRole("article");

  pane.open("second");
  fireEvent.click(await threadPane().findByRole("link"));

  expect(pane.linksOpened()).toEqual(["https://vercel.com/logs"]);
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
