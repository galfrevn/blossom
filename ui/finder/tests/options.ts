import { expect, test, vi } from "vitest";
import type { Command } from "../../commands";
import type { SearchResults } from "../../core/contract";
import { type FinderOption, groupsOf, optionsOf } from "../options";

function command(title: string, group: Command["group"] = "lenses"): Command {
  return { id: title, group, title, shortcut: "2", run: vi.fn() };
}

const commands = [
  command("Go to Flow"),
  command("Go to Tasks"),
  command("Show the whole board", "actions"),
];

const foundOneMessage: SearchResults = {
  channels: [],
  tasks: [],
  messages: [
    {
      id: "47249",
      channel: "fleet/www",
      author: "claude",
      createdAt: "2026-11-03T12:00:00.000Z",
      snippet: [{ text: "the flow of builds", marked: false }],
    },
  ],
  agents: [],
};

const nothingFound: SearchResults = {
  channels: [],
  tasks: [],
  messages: [],
  agents: [],
};

function described(options: FinderOption[]) {
  return options.map((option) => {
    if (option.kind === "command") {
      return option.command.title;
    }
    return option.kind;
  });
}

test("with nothing typed, every command is listed, in the order declared", () => {
  expect(described(optionsOf(commands, "", nothingFound))).toEqual([
    "Go to Flow",
    "Go to Tasks",
    "Show the whole board",
  ]);
});

test("commands that match well come before what the search found", () => {
  const options = optionsOf(commands, "fl", foundOneMessage);

  expect(described(options)).toEqual(["Go to Flow", "message"]);
});

test("commands that match only inside a word come after what the search found", () => {
  const options = optionsOf(commands, "low", foundOneMessage);

  expect(described(options)).toEqual(["message", "Go to Flow"]);
});

test("a command carries its title with the words typed marked", () => {
  const [option] = optionsOf(commands, "tas", nothingFound);

  expect(option?.kind === "command" && option.marked).toEqual([
    { text: "Go to ", marked: false },
    { text: "Tas", marked: true },
    { text: "ks", marked: false },
  ]);
});

test("commands are grouped as they are declared, results by what they are", () => {
  const options = optionsOf(commands, "", foundOneMessage);

  const groups = groupsOf(options).map((group) => ({
    name: group.name,
    firstIndex: group.firstIndex,
    count: group.options.length,
  }));

  expect(groups).toEqual([
    { name: "lenses", firstIndex: 0, count: 2 },
    { name: "actions", firstIndex: 2, count: 1 },
    { name: "message", firstIndex: 3, count: 1 },
  ]);
});
