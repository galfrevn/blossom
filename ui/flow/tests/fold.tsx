import { act, cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import type { FlowRow } from "../../core/contract";
import { dictionary } from "../../dictionary";
import { FlowLine } from "../line";

afterEach(cleanup);

const TWELVE_UPDATES = 12;
const THREE_AGENTS = 3;

function progressRow(id: string, text: string): FlowRow {
  return {
    id,
    channel: "fleet/www",
    author: "claude",
    onBehalfOf: "ada",
    createdAt: "2026-11-21T11:00:00.000Z",
    text,
    mentions: [],
    edited: false,
    deleted: false,
    firstOfItsMinute: false,
    fold: null,
  };
}

function foldOf(recent: FlowRow[]): FlowRow {
  const latest = recent.at(-1) ?? progressRow("0", "");
  return {
    ...latest,
    fold: { count: TWELVE_UPDATES, agents: THREE_AGENTS, recent },
  };
}

const recentLines = [
  progressRow("11", "installing dependencies"),
  progressRow("12", "running the tests"),
  progressRow("13", "tests pass, opening a pull request"),
];

test("a fold reads as one line: its channel, how many updates from how many agents, and the latest", () => {
  render(<FlowLine row={foldOf(recentLines)} selection={undefined} />);

  const line = screen.getByRole("button", { expanded: false });
  expect(within(line).getByText("#fleet/www")).toBeDefined();
  expect(
    within(line).getByText(
      dictionary.flow.foldCount(TWELVE_UPDATES, THREE_AGENTS),
    ),
  ).toBeDefined();
  expect(
    within(line).getByText(
      dictionary.flow.latest("tests pass, opening a pull request"),
    ),
  ).toBeDefined();
  expect(screen.queryByText("installing dependencies")).toBeNull();
});

test("opening a fold shows its last messages in place, and closing it hides them", () => {
  render(<FlowLine row={foldOf(recentLines)} selection={undefined} />);

  act(() => screen.getByRole("button", { expanded: false }).click());
  expect(screen.getByText("installing dependencies")).toBeDefined();
  expect(screen.getByText("running the tests")).toBeDefined();

  act(() => screen.getByRole("button", { expanded: true }).click());
  expect(screen.queryByText("installing dependencies")).toBeNull();
});

test("one update from one agent says so in the singular", () => {
  expect(dictionary.flow.foldCount(1, 1)).toBe(
    "1 progress update from 1 agent",
  );
});

test("a fold that starts a minute has that minute's divider above it", () => {
  render(
    <FlowLine
      row={{ ...foldOf(recentLines), firstOfItsMinute: true }}
      selection={undefined}
    />,
  );

  expect(screen.getByRole("heading", { level: 3 }).textContent).toBe(
    dictionary.flow.time("2026-11-21T11:00:00.000Z"),
  );
});

test("a deleted update reads as deleted, in the fold's line and when it is open", () => {
  const deletedLatest = {
    ...progressRow("14", "secret token pasted by mistake"),
    deleted: true,
  };
  render(
    <FlowLine
      row={foldOf([...recentLines, deletedLatest])}
      selection={undefined}
    />,
  );

  expect(
    screen.getByText(dictionary.flow.latest(dictionary.flow.deleted)),
  ).toBeDefined();
  act(() => screen.getByRole("button", { expanded: false }).click());
  expect(screen.queryByText("secret token pasted by mistake")).toBeNull();
});

test("a closed fold reads as chosen while its latest update's thread is open", () => {
  render(<FlowLine row={foldOf(recentLines)} selection="13" />);

  expect(
    screen
      .getByRole("button", { expanded: false })
      .getAttribute("aria-current"),
  ).toBe("true");
});

test("an open fold shows each update as a line of its own, which can be chosen", () => {
  render(<FlowLine row={foldOf(recentLines)} selection="12" />);
  act(() => screen.getByRole("button", { expanded: false }).click());

  const updates = screen.getAllByRole("article");
  expect(updates.map((update) => update.dataset.message)).toEqual([
    "11",
    "12",
    "13",
  ]);
  expect(updates.map((update) => update.getAttribute("aria-current"))).toEqual([
    "false",
    "true",
    "false",
  ]);
  expect(
    screen.getByRole("button", { expanded: true }).getAttribute("aria-current"),
  ).toBe("false");
});
