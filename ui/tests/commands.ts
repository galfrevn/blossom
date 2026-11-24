import { expect, test, vi } from "vitest";
import { type Command, commandsOf } from "../commands";
import { dictionary } from "../dictionary";
import { LENSES } from "../frame/lenses";

function actions() {
  return { showLens: vi.fn(), widen: vi.fn(), showShortcuts: vi.fn() };
}

function commandTitled(commands: Command[], title: string): Command {
  const command = commands.find((candidate) => candidate.title === title);
  if (command === undefined) {
    throw new Error(`no command is titled ${title}`);
  }
  return command;
}

test("every lens has a command that goes to it, with the lens's key, in the sidebar's order", () => {
  const commands = commandsOf({ lens: "flow", channel: null }, actions());

  expect(
    commands
      .filter((command) => command.group === "lenses")
      .map(({ group, title, shortcut }) => ({ group, title, shortcut })),
  ).toEqual(
    LENSES.map((entry) => ({
      group: "lenses",
      title: dictionary.commands.goTo(entry.name),
      shortcut: entry.key,
    })),
  );
});

test("going to a lens shows it", () => {
  const spies = actions();
  const commands = commandsOf({ lens: "flow", channel: null }, spies);

  commandTitled(commands, dictionary.commands.goTo("Tasks")).run();

  expect(spies.showLens).toHaveBeenCalledWith("tasks");
});

test("narrowed to a channel, a command shows the whole board, with the key of the lens on screen", () => {
  const spies = actions();
  const commands = commandsOf({ lens: "tasks", channel: "fleet/www" }, spies);
  const widening = commandTitled(commands, dictionary.lenses.widen);

  widening.run();

  expect(widening.group).toBe("actions");
  expect(widening.shortcut).toBe("3");
  expect(spies.widen).toHaveBeenCalledOnce();
});

test("with the whole board on screen, no command widens it", () => {
  const commands = commandsOf({ lens: "flow", channel: null }, actions());

  expect(commands.map((command) => command.title)).not.toContain(
    dictionary.lenses.widen,
  );
});

test("a command shows the sheet of shortcuts, with ?", () => {
  const spies = actions();
  const commands = commandsOf({ lens: "flow", channel: null }, spies);
  const showing = commandTitled(commands, dictionary.commands.showShortcuts);

  showing.run();

  expect(showing.group).toBe("actions");
  expect(showing.shortcut).toBe("?");
  expect(spies.showShortcuts).toHaveBeenCalledOnce();
});
