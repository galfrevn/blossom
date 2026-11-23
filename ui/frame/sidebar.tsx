import { useCallback, useEffect, useId } from "react";
import type { TaskChange, TaskCount } from "../core/contract";
import { dictionary } from "../dictionary";
import type { Lens } from "./choice";
import { CountBeside, type TaskCounting } from "./count";
import { LENSES, type LensEntry } from "./lenses";

const INDENT_FOR_EACH_DEPTH = [
  "pl-2",
  "pl-5",
  "pl-8",
  "pl-11",
  "pl-14",
  "pl-17",
];
const DEEPEST_INDENT = INDENT_FOR_EACH_DEPTH.length - 1;

type ChannelRow = {
  path: string;
  name: string;
  depth: number;
  isChannel: boolean;
};

const channelNameOrder = new Intl.Collator("en", { numeric: true });

function compareChannelPaths(left: string, right: string): number {
  const leftParts = left.split("/");
  const rightParts = right.split("/");
  const sharedLength = Math.min(leftParts.length, rightParts.length);
  for (let index = 0; index < sharedLength; index += 1) {
    const order = channelNameOrder.compare(
      leftParts[index] ?? "",
      rightParts[index] ?? "",
    );
    if (order !== 0) {
      return order;
    }
  }
  return leftParts.length - rightParts.length;
}

function rowsOfTheTree(channels: string[]): ChannelRow[] {
  const known = new Set(channels);
  const listed = new Set<string>();
  const rows: ChannelRow[] = [];
  for (const channel of [...channels].sort(compareChannelPaths)) {
    const parts = channel.split("/");
    for (let depth = 0; depth < parts.length; depth += 1) {
      const path = parts.slice(0, depth + 1).join("/");
      if (!listed.has(path)) {
        listed.add(path);
        rows.push({
          path,
          name: parts[depth] ?? path,
          depth,
          isChannel: known.has(path),
        });
      }
    }
  }
  return rows;
}

function labelOf(row: ChannelRow): string {
  if (row.isChannel) {
    return dictionary.sidebar.channel(row.name);
  }
  return row.name;
}

function indentOf(depth: number): string {
  return INDENT_FOR_EACH_DEPTH[Math.min(depth, DEEPEST_INDENT)] ?? "pl-2";
}

const LENS_ICON =
  "size-[14px] flex-none fill-none stroke-current [stroke-linecap:round] [stroke-linejoin:round] [stroke-width:1.3]";

function FlowIcon() {
  return (
    <svg viewBox="0 0 14 14" aria-hidden={true} className={LENS_ICON}>
      <path d="M1.5 3.5h11M1.5 7h8M1.5 10.5h10" />
    </svg>
  );
}

function TasksIcon() {
  return (
    <svg viewBox="0 0 14 14" aria-hidden={true} className={LENS_ICON}>
      <rect x="1.5" y="1.5" width="11" height="11" rx="1" />
      <path d="M4 5h6M4 7.5h6M4 10h3" />
    </svg>
  );
}

const ICON_OF_EACH_LENS: Record<Lens, () => React.ReactNode> = {
  flow: FlowIcon,
  tasks: TasksIcon,
};

function isTypingInto(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }
  const takesText =
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement;
  return takesText || target.isContentEditable;
}

function useLensKeys(onChooseLens: (lens: Lens) => void) {
  useEffect(() => {
    const chooseTheLensOfTheKey = (event: KeyboardEvent) => {
      const pressedWithModifiers =
        event.metaKey || event.ctrlKey || event.altKey || event.shiftKey;
      if (pressedWithModifiers || isTypingInto(event.target)) {
        return;
      }
      const entry = LENSES.find((candidate) => candidate.key === event.key);
      if (entry !== undefined) {
        onChooseLens(entry.lens);
      }
    };
    document.addEventListener("keydown", chooseTheLensOfTheKey);
    return () => {
      document.removeEventListener("keydown", chooseTheLensOfTheKey);
    };
  }, [onChooseLens]);
}

function Heading({ id, children }: { id: string; children: string }) {
  return (
    <h2
      id={id}
      className="flex items-center gap-2.5 px-2 pt-1.5 pb-1 font-mono text-[10px] text-ink-3 uppercase leading-none tracking-[0.09em] after:h-px after:flex-1 after:bg-line"
    >
      {children}
    </h2>
  );
}

function currentnessOf(isCurrent: boolean): "true" | "false" {
  if (isCurrent) {
    return "true";
  }
  return "false";
}

function lensCurrentnessOf(isChosen: boolean): "page" | "false" {
  if (isChosen) {
    return "page";
  }
  return "false";
}

const NAVIGATION_ROW =
  "flex w-full items-center gap-[9px] rounded-[5px] py-[5px] pr-2 text-left text-[12.5px] text-ink-2 hover:bg-sunken hover:text-ink aria-[current=page]:bg-accent-wash aria-[current=page]:text-ink aria-[current=true]:bg-accent-wash aria-[current=true]:text-ink";

function ChannelButton({
  row,
  isChosen,
  onChoose,
}: {
  row: ChannelRow;
  isChosen: boolean;
  onChoose: (channel: string) => void;
}) {
  const choose = useCallback(() => onChoose(row.path), [onChoose, row.path]);
  return (
    <button
      type="button"
      aria-current={currentnessOf(isChosen)}
      onClick={choose}
      className={`${NAVIGATION_ROW} ${indentOf(row.depth)}`}
    >
      <span className="min-w-0 flex-1 truncate">{labelOf(row)}</span>
    </button>
  );
}

function LensButton({
  entry,
  isChosen,
  onChoose,
  counting,
}: {
  entry: LensEntry;
  isChosen: boolean;
  onChoose: (lens: Lens) => void;
  counting: TaskCounting;
}) {
  const choose = useCallback(
    () => onChoose(entry.lens),
    [onChoose, entry.lens],
  );
  const countId = useId();
  const Icon = ICON_OF_EACH_LENS[entry.lens];
  return (
    <button
      type="button"
      aria-label={entry.name}
      aria-describedby={countId}
      aria-current={lensCurrentnessOf(isChosen)}
      aria-keyshortcuts={entry.key}
      onClick={choose}
      className={`${NAVIGATION_ROW} pl-2`}
    >
      <Icon />
      <span className="min-w-0 flex-1 truncate">{entry.name}</span>
      <CountBeside lens={entry.lens} id={countId} counting={counting} />
      <kbd className="min-w-4 rounded-[3px] border border-line bg-raised px-1 py-0.5 text-center font-mono text-[10px] text-ink-3 leading-none">
        {entry.key}
      </kbd>
    </button>
  );
}

export function Sidebar({
  channels,
  chosenChannel,
  chosenLens,
  onChooseChannel,
  onChooseLens,
  taskChange,
  readTaskCounts,
}: {
  channels: string[];
  chosenChannel: string | null;
  chosenLens: Lens;
  onChooseChannel: (channel: string) => void;
  onChooseLens: (lens: Lens) => void;
  taskChange: TaskChange | undefined;
  readTaskCounts: (channel: string | null) => Promise<TaskCount[]>;
}) {
  const text = dictionary.sidebar;
  const counting: TaskCounting = {
    channel: chosenChannel,
    change: taskChange,
    readTaskCounts,
  };
  const lensesHeading = useId();
  const channelsHeading = useId();
  useLensKeys(onChooseLens);
  return (
    <aside className="flex w-[212px] shrink-0 flex-col gap-3.5 overflow-auto border-line border-r bg-surface px-1.5 pt-2 pb-6">
      <section>
        <Heading id={lensesHeading}>{text.lenses}</Heading>
        <ul aria-labelledby={lensesHeading} className="flex flex-col gap-px">
          {LENSES.map((entry) => (
            <li key={entry.lens}>
              <LensButton
                entry={entry}
                isChosen={entry.lens === chosenLens}
                onChoose={onChooseLens}
                counting={counting}
              />
            </li>
          ))}
        </ul>
      </section>
      <section>
        <Heading id={channelsHeading}>{text.channels}</Heading>
        <ul aria-labelledby={channelsHeading} className="flex flex-col gap-px">
          {rowsOfTheTree(channels).map((row) => (
            <li key={row.path} data-depth={row.depth}>
              <ChannelButton
                row={row}
                isChosen={row.path === chosenChannel}
                onChoose={onChooseChannel}
              />
            </li>
          ))}
        </ul>
      </section>
    </aside>
  );
}
