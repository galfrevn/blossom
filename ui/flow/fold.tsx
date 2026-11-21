import { useCallback, useState } from "react";
import type { FlowFold, FlowRow } from "../core/contract";
import { dictionary } from "../dictionary";
import { textOf } from "./text";

function expandednessOf(open: boolean): "true" | "false" {
  if (open) {
    return "true";
  }
  return "false";
}

function RecentLines({
  fold,
  open,
  selection,
}: {
  fold: FlowFold;
  open: boolean;
  selection: string | undefined;
}) {
  if (!open) {
    return null;
  }
  return (
    <div className="mr-5 mb-1 ml-[76px] flex flex-col border-line border-l">
      {fold.recent.map((line) => (
        <article
          key={line.id}
          data-message={line.id}
          aria-current={selection === line.id}
          className="flex gap-2 py-0.5 pl-3 text-ink-2 hover:bg-raised aria-[current=true]:bg-accent-wash"
        >
          <b className="shrink-0 font-mono font-normal text-[11.5px] text-ink-3">
            {line.author}
          </b>
          <span className="min-w-0 [overflow-wrap:anywhere]">
            {textOf(line)}
          </span>
        </article>
      ))}
    </div>
  );
}

export function FoldLine({
  row,
  fold,
  selection,
}: {
  row: FlowRow;
  fold: FlowFold;
  selection: string | undefined;
}) {
  const [open, setOpen] = useState(false);
  const toggle = useCallback(() => setOpen((wasOpen) => !wasOpen), []);
  const time = dictionary.flow.time(row.createdAt);
  const chosenWhileClosed = !open && selection === row.id;
  return (
    <div>
      <button
        type="button"
        aria-expanded={expandednessOf(open)}
        aria-current={chosenWhileClosed}
        onClick={toggle}
        className="grid w-full grid-cols-[44px_minmax(0,1fr)] gap-x-3 px-5 py-[5px] text-left text-ink-3 hover:bg-raised aria-[current=true]:bg-accent-wash"
      >
        <time
          dateTime={row.createdAt}
          className="font-mono text-[11px] leading-[1.8]"
        >
          {time}
        </time>
        <span className="flex min-w-0 flex-wrap items-baseline gap-x-2.5 gap-y-1">
          <b className="font-mono font-normal text-[11.5px] text-ink-2">
            {dictionary.flow.channel(row.channel)}
          </b>
          <span>{dictionary.flow.foldCount(fold.count, fold.agents)}</span>
          <span className="min-w-0 truncate">
            {dictionary.flow.latest(textOf(row))}
          </span>
        </span>
      </button>
      <RecentLines fold={fold} open={open} selection={selection} />
    </div>
  );
}
