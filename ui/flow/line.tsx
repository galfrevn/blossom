import type { FlowRow } from "../core/contract";
import { dictionary } from "../dictionary";

function textOf(row: FlowRow): string {
  if (row.deleted) {
    return dictionary.flow.deleted;
  }
  return row.text;
}

function MinuteDivider({ row, time }: { row: FlowRow; time: string }) {
  if (!row.firstOfItsMinute) {
    return null;
  }
  return (
    <div className="col-span-2 flex items-center gap-2.5 pt-2 pb-1 font-mono text-[10.5px] text-ink-3 after:h-px after:flex-1 after:bg-line">
      <time dateTime={row.createdAt}>{time}</time>
    </div>
  );
}

function OnBehalfOf({ person }: { person: string | null }) {
  if (person === null) {
    return null;
  }
  return (
    <small className="ml-1 text-[12px] text-ink-3">
      {dictionary.flow.onBehalfOf(person)}
    </small>
  );
}

function Edited({ row }: { row: FlowRow }) {
  if (!row.edited) {
    return null;
  }
  return (
    <span className="ml-1.5 text-[11px] text-ink-3">
      {dictionary.flow.edited}
    </span>
  );
}

export function FlowLine({ row }: { row: FlowRow }) {
  const time = dictionary.flow.time(row.createdAt);
  return (
    <article
      data-message={row.id}
      className="grid grid-cols-[44px_minmax(0,1fr)] gap-x-3 px-5 py-[5px] hover:bg-raised"
    >
      <MinuteDivider row={row} time={time} />
      <time
        dateTime={row.createdAt}
        className="font-mono text-[11px] text-ink-3 leading-[1.8]"
      >
        {time}
      </time>
      <div className="min-w-0">
        <span className="font-medium">{row.author}</span>
        <OnBehalfOf person={row.onBehalfOf} />
        <span className="ml-1.5 font-mono text-[11px] text-ink-3">
          {dictionary.flow.channel(row.channel)}
        </span>
        <Edited row={row} />
        <p className="line-clamp-4 whitespace-pre-line text-ink-2 [overflow-wrap:anywhere]">
          {textOf(row)}
        </p>
      </div>
    </article>
  );
}
