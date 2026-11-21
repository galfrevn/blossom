import { act, cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import type { FlowChange, FlowRow } from "../../core/contract";
import { createFakeCore } from "../../core/fake";
import { dictionary } from "../../dictionary";
import { FlowList } from "../list";

afterEach(cleanup);

const SOMEONE_ACTING_FOR_A_PERSON = /^via /u;

function renderFlowOf(rows: FlowRow[]) {
  const fakeCore = createFakeCore();
  let latestChange: FlowChange | undefined;
  fakeCore.core.subscribeToFlow((change) => {
    latestChange = change;
  });
  fakeCore.pushFlow(rows);
  const { rerender } = render(
    <FlowList change={latestChange} readFlow={fakeCore.core.readFlow} />,
  );
  return {
    pushFlow: (newRows: FlowRow[]) => {
      fakeCore.pushFlow(newRows);
      act(() => {
        rerender(
          <FlowList change={latestChange} readFlow={fakeCore.core.readFlow} />,
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
