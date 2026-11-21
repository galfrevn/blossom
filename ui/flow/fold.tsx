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

function RecentLines({ fold, open }: { fold: FlowFold; open: boolean }) {
  if (!open) {
    return null;
  }
  return (
    <span className="col-span-2 mt-1 flex flex-col gap-0.5 border-line border-l pl-3">
      {fold.recent.map((line) => (
        <span key={line.id} className="flex gap-2 text-ink-2">
          <b className="shrink-0 font-mono font-normal text-[11.5px] text-ink-3">
            {line.author}
          </b>
          <span className="min-w-0 [overflow-wrap:anywhere]">
            {textOf(line)}
          </span>
        </span>
      ))}
    </span>
  );
}

export function FoldLine({ row, fold }: { row: FlowRow; fold: FlowFold }) {
  const [open, setOpen] = useState(false);
  const toggle = useCallback(() => setOpen((wasOpen) => !wasOpen), []);
  const time = dictionary.flow.time(row.createdAt);
  return (
    <button
      type="button"
      aria-expanded={expandednessOf(open)}
      onClick={toggle}
      className="grid w-full grid-cols-[44px_minmax(0,1fr)] gap-x-3 px-5 py-[5px] text-left text-ink-3 hover:bg-raised"
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
      <RecentLines fold={fold} open={open} />
    </button>
  );
}
