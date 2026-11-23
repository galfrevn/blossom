import { act, renderHook, waitFor } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import type { Command } from "../../commands";
import type { SearchResults } from "../../core/contract";
import { createFakeCore } from "../../core/fake";
import { useFinding } from "../finding";
import { type FinderOption, optionsOf } from "../options";

function command(title: string): Command {
  return { id: title, group: "lenses", title, shortcut: "2", run: vi.fn() };
}

const commands = [command("Go to Flow"), command("Go to Tasks")];

const foundTasksMessage: SearchResults = {
  channels: [],
  tasks: [],
  messages: [
    {
      id: "47249",
      channel: "fleet/www",
      author: "claude",
      createdAt: "2026-11-03T12:00:00.000Z",
      snippet: [{ text: "tasks are blocked", marked: false }],
    },
  ],
  agents: [],
};

function renderFinding() {
  const fakeCore = createFakeCore();
  fakeCore.answerSearch("tas", foundTasksMessage);
  return renderHook(() => useFinding(fakeCore.core.search, commands));
}

function titlesOf(options: FinderOption[]) {
  return options.map((option) => {
    if (option.kind === "command") {
      return option.command.title;
    }
    return option.kind;
  });
}

test("before anything is typed, every command is listed and the first is selected", () => {
  const { result } = renderFinding();

  expect(titlesOf(result.current.options)).toEqual([
    "Go to Flow",
    "Go to Tasks",
  ]);
  expect(result.current.selectedIndex).toBe(0);
  expect(result.current.listShown).toBe(true);
});

test("the commands follow what is typed at once, before the search answers", () => {
  const { result } = renderFinding();

  act(() => {
    result.current.typeQuery("tas");
  });

  expect(titlesOf(result.current.options)).toEqual(["Go to Tasks"]);
});

test("what the search found joins the commands once it answers", async () => {
  const { result } = renderFinding();

  act(() => {
    result.current.typeQuery("tas");
  });

  await waitFor(() => {
    expect(titlesOf(result.current.options)).toEqual(
      titlesOf(optionsOf(commands, "tas", foundTasksMessage)),
    );
  });
});

test("typing selects the first option again", () => {
  const { result } = renderFinding();

  act(() => {
    result.current.moveSelection(1);
  });
  const moved = result.current.selectedIndex;
  act(() => {
    result.current.typeQuery("go");
  });

  expect(moved).toBe(1);
  expect(result.current.selectedIndex).toBe(0);
});

test("with no commands and nothing typed, nothing is listed", () => {
  const fakeCore = createFakeCore();
  const { result } = renderHook(() => useFinding(fakeCore.core.search, []));

  expect(result.current.listShown).toBe(false);
});
