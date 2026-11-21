import { useId } from "react";
import { dictionary } from "../dictionary";

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

function FlowIcon() {
  return (
    <svg
      viewBox="0 0 14 14"
      aria-hidden={true}
      className="size-[14px] flex-none fill-none stroke-current [stroke-linecap:round] [stroke-width:1.3]"
    >
      <path d="M1.5 3.5h11M1.5 7h8M1.5 10.5h10" />
    </svg>
  );
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

export function Sidebar({ channels }: { channels: string[] }) {
  const text = dictionary.sidebar;
  const lensesHeading = useId();
  const channelsHeading = useId();
  return (
    <aside className="flex w-[212px] shrink-0 flex-col gap-3.5 overflow-auto border-line border-r bg-surface px-1.5 pt-2 pb-6">
      <section>
        <Heading id={lensesHeading}>{text.lenses}</Heading>
        <ul aria-labelledby={lensesHeading} className="flex flex-col gap-px">
          <li
            aria-current="page"
            className="flex items-center gap-[9px] rounded-[5px] bg-accent-wash px-2 py-[5px] text-[12.5px] text-ink"
          >
            <FlowIcon />
            <span className="min-w-0 flex-1 truncate">
              {dictionary.lenses.flow.title}
            </span>
          </li>
        </ul>
      </section>
      <section>
        <Heading id={channelsHeading}>{text.channels}</Heading>
        <ul aria-labelledby={channelsHeading} className="flex flex-col gap-px">
          {rowsOfTheTree(channels).map((row) => (
            <li
              key={row.path}
              data-depth={row.depth}
              className={`truncate rounded-[5px] py-[5px] pr-2 text-[12.5px] text-ink-2 ${indentOf(row.depth)}`}
            >
              {labelOf(row)}
            </li>
          ))}
        </ul>
      </section>
    </aside>
  );
}
