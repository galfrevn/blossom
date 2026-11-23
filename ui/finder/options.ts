import type { Command } from "../commands";
import type {
  FoundChannel,
  FoundMessage,
  FoundTask,
  SearchResults,
  TextPart,
} from "../core/contract";
import { matchOf } from "./matching";

type CommandOption = { kind: "command"; command: Command; marked: TextPart[] };

type Matched = {
  well: CommandOption[];
  weakly: CommandOption[];
};

function commandsMatching(commands: Command[], query: string): Matched {
  const matched: Matched = { well: [], weakly: [] };
  for (const command of commands) {
    const match = matchOf(command.title, query);
    if (match !== undefined) {
      matched[match.quality].push({
        kind: "command",
        command,
        marked: match.marked,
      });
    }
  }
  return matched;
}

function resultOptionsOf(results: SearchResults): FinderOption[] {
  const channels = results.channels.map(
    (found): FinderOption => ({ kind: "channel", found }),
  );
  const tasks = results.tasks.map(
    (found): FinderOption => ({ kind: "task", found }),
  );
  const messages = results.messages.map(
    (found): FinderOption => ({ kind: "message", found }),
  );
  return [...channels, ...tasks, ...messages];
}

function groupNameOf(option: FinderOption): GroupName {
  if (option.kind === "command") {
    return option.command.group;
  }
  return option.kind;
}

export type FinderOption =
  | CommandOption
  | { kind: "channel"; found: FoundChannel }
  | { kind: "task"; found: FoundTask }
  | { kind: "message"; found: FoundMessage };

export type GroupName = Command["group"] | "channel" | "task" | "message";

export type FinderGroup = {
  name: GroupName;
  firstIndex: number;
  options: FinderOption[];
};

export function optionsOf(
  commands: Command[],
  query: string,
  results: SearchResults,
): FinderOption[] {
  const matched = commandsMatching(commands, query);
  return [...matched.well, ...resultOptionsOf(results), ...matched.weakly];
}

export function groupsOf(options: FinderOption[]): FinderGroup[] {
  const groups: FinderGroup[] = [];
  for (const [index, option] of options.entries()) {
    const name = groupNameOf(option);
    const group = groups.at(-1);
    if (group?.name === name) {
      group.options.push(option);
    } else {
      groups.push({ name, firstIndex: index, options: [option] });
    }
  }
  return groups;
}

export function keyOf(option: FinderOption): string {
  if (option.kind === "command") {
    return `command:${option.command.id}`;
  }
  if (option.kind === "channel") {
    return `channel:${option.found.path}`;
  }
  return `${option.kind}:${option.found.id}`;
}
