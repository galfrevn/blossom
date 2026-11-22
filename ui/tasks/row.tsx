import { memo } from "react";
import { type Clock, useReadingOf } from "../core/clock";
import type { TaskRow } from "../core/contract";
import { dictionary } from "../dictionary";
import {
  COLUMNS,
  DROPPED_WHEN_NARROW,
  DROPPED_WHEN_NARROWER,
  DROPPED_WHEN_NARROWEST,
} from "./columns";
import { LeaseLeft } from "./lease";
import { looksOf } from "./states";

const MILLISECONDS_IN_A_SECOND = 1000;
const MONO_CELL = "min-w-0 truncate font-mono text-[11.5px] text-ink-3";

function StatePill({ state }: { state: string }) {
  const looks = looksOf(state);
  return (
    <span
      className={`inline-flex items-center gap-1.5 whitespace-nowrap font-medium font-mono text-[10.5px] uppercase leading-[1.6] tracking-[0.06em] ${looks.ink}`}
    >
      <i
        aria-hidden={true}
        className={`size-[7px] flex-none rounded-[1px] ${looks.swatch}`}
      />
      {looks.name}
    </span>
  );
}

function Lease({ row, clock }: { row: TaskRow; clock: Clock }) {
  if (row.lease === null) {
    return null;
  }
  return <LeaseLeft lease={row.lease} clock={clock} />;
}

function Holder({ row, clock }: { row: TaskRow; clock: Clock }) {
  if (row.holder === null) {
    return null;
  }
  return (
    <>
      <span className="min-w-0 truncate">{row.holder}</span>
      <Lease row={row} clock={clock} />
    </>
  );
}

function secondsSince(changedAt: string, now: number): number {
  const milliseconds = now - Date.parse(changedAt);
  return Math.max(0, Math.floor(milliseconds / MILLISECONDS_IN_A_SECOND));
}

function TimeSinceTheChange({
  changedAt,
  clock,
}: {
  changedAt: string;
  clock: Clock;
}) {
  const timeSince = useReadingOf(clock, (now) =>
    dictionary.tasks.timeSince(secondsSince(changedAt, now)),
  );
  return <time dateTime={changedAt}>{timeSince}</time>;
}

export const TaskLine = memo(function TaskLineOnScreen({
  index,
  setSize,
  row,
  selected,
  clock,
}: {
  index: number;
  setSize: number;
  row: TaskRow;
  selected: boolean;
  clock: Clock;
}) {
  return (
    <li
      data-index={index}
      data-task={row.id}
      aria-posinset={index + 1}
      aria-setsize={setSize}
      aria-current={selected}
      className={`absolute inset-x-0 top-0 h-[30px] border-line border-b text-[12.5px] hover:bg-raised aria-[current=true]:bg-accent-wash ${COLUMNS}`}
    >
      <StatePill state={row.state} />
      <span className="min-w-0 truncate">{row.title}</span>
      <span className={`${MONO_CELL} ${DROPPED_WHEN_NARROWER}`}>
        {dictionary.tasks.channel(row.channel)}
      </span>
      <span className={`${MONO_CELL} ${DROPPED_WHEN_NARROW}`}>{row.owner}</span>
      <span
        className={`flex min-w-0 items-center gap-2 text-ink-2 ${DROPPED_WHEN_NARROWEST}`}
      >
        <Holder row={row} clock={clock} />
      </span>
      <span
        className={`whitespace-nowrap font-mono text-[11.5px] text-ink-3 ${DROPPED_WHEN_NARROW}`}
      >
        <TimeSinceTheChange changedAt={row.changedAt} clock={clock} />
      </span>
    </li>
  );
});
