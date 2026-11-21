import type { FlowRow } from "../core/contract";
import { dictionary } from "../dictionary";
import { FoldLine } from "./fold";
import { textOf } from "./text";

function MinuteDivider({ row }: { row: FlowRow }) {
  if (!row.firstOfItsMinute) {
    return null;
  }
  return (
    <h3 className="flex items-center gap-2.5 px-5 pt-3 pb-1 font-mono font-normal text-[10.5px] text-ink-3 after:h-px after:flex-1 after:bg-line">
      <time dateTime={row.createdAt}>
        {dictionary.flow.time(row.createdAt)}
      </time>
    </h3>
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

function MessageLine({ row, selected }: { row: FlowRow; selected: boolean }) {
  const time = dictionary.flow.time(row.createdAt);
  return (
    <article
      data-message={row.id}
      aria-current={selected}
      className="grid grid-cols-[44px_minmax(0,1fr)] gap-x-3 px-5 py-[5px] hover:bg-raised aria-[current=true]:bg-accent-wash"
    >
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

function LineOf({
  row,
  selection,
}: {
  row: FlowRow;
  selection: string | undefined;
}) {
  if (row.fold !== null) {
    return <FoldLine row={row} fold={row.fold} selection={selection} />;
  }
  return <MessageLine row={row} selected={selection === row.id} />;
}

export function FlowLine({
  row,
  selection,
}: {
  row: FlowRow;
  selection: string | undefined;
}) {
  return (
    <>
      <MinuteDivider row={row} />
      <LineOf row={row} selection={selection} />
    </>
  );
}
