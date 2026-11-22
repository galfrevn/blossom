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

const SOMEONE_ACTING_FOR_A_PERSON = /^via /u;
const BOARD_URL = "http://127.0.0.1:4200/api/board";

function renderFlowOf(rows: FlowRow[], channel: string | null = null) {
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
      channel={channel}
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

test("a message that starts a minute has that minute's divider above it", async () => {
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

  await flowList().findAllByRole("article");
  const dividers = flowList().getAllByRole("heading", { level: 3 });

  expect(dividers.map((divider) => divider.textContent)).toEqual([
    dictionary.flow.time("2026-11-21T10:00:05.000Z"),
  ]);
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
