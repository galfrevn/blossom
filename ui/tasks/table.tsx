import { useEffect, useLayoutEffect, useReducer, useRef } from "react";
import type { Clock } from "../core/clock";
import type { TaskChange, TaskNarrowing } from "../core/contract";
import { dictionary } from "../dictionary";
import {
  COLUMNS,
  DROPPED_WHEN_NARROW,
  DROPPED_WHEN_NARROWER,
  DROPPED_WHEN_NARROWEST,
} from "./columns";
import { LeaseDither } from "./lease";
import { TaskLine } from "./row";
import {
  followTheChange,
  followTheNarrowing,
  type NarrowedRows,
  narrowsAnything,
  type ReadTasks,
  ROW_HEIGHT,
  rangeOnScreen,
  rangeToRead,
  shownRows,
  type TableState,
  useTableState,
} from "./rows";
import { useSelectionByArrows, useSelectionByClicks } from "./selection";

type Elements = {
  scroller: React.RefObject<HTMLDivElement | null>;
  rows: React.RefObject<HTMLOListElement | null>;
};

function useScrollFollowing(
  { scroller }: Elements,
  state: TableState,
  redraw: () => void,
) {
  useEffect(() => {
    const element = scroller.current;
    if (element === null) {
      return;
    }
    let frame = 0;
    const recordTheScrollPosition = () => {
      frame = 0;
      state.scrollTop = element.scrollTop;
      state.viewportHeight = element.clientHeight;
      redraw();
    };
    const onScroll = () => {
      if (frame === 0) {
        frame = requestAnimationFrame(recordTheScrollPosition);
      }
    };
    const resizing = new ResizeObserver(() => {
      state.viewportHeight = element.clientHeight;
      redraw();
    });
    state.viewportHeight = element.clientHeight;
    element.addEventListener("scroll", onScroll, { passive: true });
    resizing.observe(element);
    return () => {
      cancelAnimationFrame(frame);
      resizing.disconnect();
      element.removeEventListener("scroll", onScroll);
    };
  }, [scroller, state, redraw]);
}

function useLayoutAfterDrawing(
  { scroller, rows }: Elements,
  state: TableState,
  total: number,
) {
  useLayoutEffect(() => {
    const scrollerElement = scroller.current;
    const rowsElement = rows.current;
    if (scrollerElement === null || rowsElement === null) {
      return;
    }
    rowsElement.style.height = `${total * ROW_HEIGHT}px`;
    for (const row of rowsElement.querySelectorAll<HTMLElement>(
      "[data-index]",
    )) {
      const top = Number(row.dataset.index) * ROW_HEIGHT;
      row.style.transform = `translateY(${top}px)`;
    }
    if (state.backToTheTop) {
      state.backToTheTop = false;
      scrollerElement.scrollTop = 0;
    }
  });
}

function HeaderRow() {
  const { columns } = dictionary.tasks;
  return (
    <div
      className={`sticky top-0 z-[1] h-[28px] border-line border-b bg-paper font-medium font-mono text-[10px] text-ink-3 uppercase tracking-[0.08em] ${COLUMNS}`}
    >
      <span>{columns.state}</span>
      <span>{columns.task}</span>
      <span className={DROPPED_WHEN_NARROWER}>{columns.channel}</span>
      <span className={DROPPED_WHEN_NARROW}>{columns.owner}</span>
      <span className={DROPPED_WHEN_NARROWEST}>{columns.heldBy}</span>
      <span className={DROPPED_WHEN_NARROW}>{columns.updated}</span>
    </div>
  );
}

function sayingWhenNothingIsListed(shown: NarrowedRows): string {
  if (narrowsAnything(shown.narrowing)) {
    return dictionary.tasks.noneMatch;
  }
  return dictionary.tasks.noTasks;
}

function NothingToList({ shown }: { shown: NarrowedRows }) {
  if (shown.windows.total() !== 0) {
    return null;
  }
  return (
    <p className="px-5 py-6 text-[12.5px] text-ink-3">
      {sayingWhenNothingIsListed(shown)}
    </p>
  );
}

function linesBetween(
  shown: NarrowedRows,
  range: { first: number; last: number },
  selectedTask: string | undefined,
  clock: Clock,
) {
  const setSize = shown.windows.total() ?? 0;
  const lines: React.ReactNode[] = [];
  for (let index = range.first; index <= range.last; index += 1) {
    const row = shown.windows.rowAt(index);
    if (row !== undefined) {
      lines.push(
        <TaskLine
          key={index}
          index={index}
          setSize={setSize}
          row={row}
          selected={row.id === selectedTask}
          clock={clock}
        />,
      );
    }
  }
  return lines;
}

export function TaskTable({
  narrowing,
  change,
  clock,
  readTasks,
  selectedTask,
  onSelect,
}: {
  narrowing: TaskNarrowing;
  change: TaskChange | undefined;
  clock: Clock;
  readTasks: ReadTasks;
  selectedTask: string | undefined;
  onSelect: (taskId: string) => void;
}) {
  const [, redraw] = useReducer((count: number) => count + 1, 0);
  const scroller = useRef<HTMLDivElement>(null);
  const rows = useRef<HTMLOListElement>(null);
  const elements = { scroller, rows };
  const state = useTableState(narrowing, readTasks, redraw);
  followTheNarrowing(state, narrowing, readTasks, redraw);
  followTheChange(state, change);
  const shown = shownRows(state);
  const total = shown.windows.total() ?? 0;
  const range = rangeOnScreen(state, shown.windows.total());
  const reading = rangeToRead(state, shown.windows.total());
  state.latest.windows.need(
    reading.first,
    Math.max(reading.first, reading.last),
  );
  useScrollFollowing(elements, state, redraw);
  useLayoutAfterDrawing(elements, state, total);
  useSelectionByClicks(rows, onSelect);
  useSelectionByArrows(scroller, state, selectedTask, onSelect);
  return (
    <div ref={scroller} className="relative min-h-0 flex-1 overflow-auto">
      <LeaseDither />
      <HeaderRow />
      <NothingToList shown={shown} />
      <ol ref={rows} aria-label={dictionary.tasks.list} className="relative">
        {linesBetween(shown, range, selectedTask, clock)}
      </ol>
    </div>
  );
}
