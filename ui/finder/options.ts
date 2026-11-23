import type {
  FoundChannel,
  FoundMessage,
  FoundTask,
  SearchResults,
} from "../core/contract";

export type FinderOption =
  | { kind: "channel"; found: FoundChannel }
  | { kind: "task"; found: FoundTask }
  | { kind: "message"; found: FoundMessage };

export type FinderGroup = {
  kind: FinderOption["kind"];
  firstIndex: number;
  options: FinderOption[];
};

export function optionsOf(results: SearchResults): FinderOption[] {
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

export function groupsOf(options: FinderOption[]): FinderGroup[] {
  const groups: FinderGroup[] = [];
  for (const [index, option] of options.entries()) {
    const group = groups.at(-1);
    if (group?.kind === option.kind) {
      group.options.push(option);
    } else {
      groups.push({ kind: option.kind, firstIndex: index, options: [option] });
    }
  }
  return groups;
}

export function keyOf(option: FinderOption): string {
  if (option.kind === "channel") {
    return `channel:${option.found.path}`;
  }
  return `${option.kind}:${option.found.id}`;
}
