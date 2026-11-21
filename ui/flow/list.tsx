import { memo, useCallback, useReducer, useRef } from "react";
import type { FlowChange, FlowRow, FlowWindow } from "../core/contract";
import { dictionary } from "../dictionary";
import { EmptyBoard } from "./empty";
import { FlowLine } from "./line";
import { useLayoutAfterDrawing, useScrollFollowing } from "./scroll";
import {
  followTheChange,
  followTheTotal,
  type ListState,
  newBelowOf,
  rangeOnScreen,
  useListState,
} from "./state";

const Placeholder = memo(function PlaceholderRow({ index }: { index: number }) {
  return (
    <div
      data-index={index}
      data-placeholder={true}
      className="absolute inset-x-0 top-0"
    />
  );
});

const Line = memo(function LoadedLine({
  index,
  row,
}: {
  index: number;
  row: FlowRow;
}) {
  return (
    <div
      data-index={index}
      data-loaded={true}
      className="absolute inset-x-0 top-0"
    >
      <FlowLine row={row} />
    </div>
  );
});

function linesBetween(state: ListState, first: number, last: number) {
  const lines: React.ReactNode[] = [];
  for (let index = first; index <= last; index += 1) {
    const row = state.windows.rowAt(index);
    if (row === undefined) {
      lines.push(<Placeholder key={index} index={index} />);
    } else {
      lines.push(<Line key={index} index={index} row={row} />);
    }
  }
  return lines;
}

function NewBelow({
  count,
  onChoose,
}: {
  count: number;
  onChoose: () => void;
}) {
  if (count === 0) {
    return null;
  }
  return (
    <button
      type="button"
      onClick={onChoose}
      className="absolute bottom-3 left-1/2 -translate-x-1/2 rounded-full border border-accent-line bg-accent-wash px-3 py-1 font-mono text-[11px] text-accent-ink"
    >
      {dictionary.flow.newBelow(count)}
    </button>
  );
}

function EmptyWhenNothingIsWritten({
  state,
  boardUrl,
}: {
  state: ListState;
  boardUrl: string;
}) {
  if (state.windows.total() !== 0) {
    return null;
  }
  return <EmptyBoard boardUrl={boardUrl} />;
}

export function FlowList({
  change,
  channel,
  boardUrl,
  readFlow,
}: {
  change: FlowChange | undefined;
  channel: string | null;
  boardUrl: string;
  readFlow: (
    channel: string | null,
    start: number,
    count: number,
  ) => Promise<FlowWindow>;
}) {
  const [, redraw] = useReducer((count: number) => count + 1, 0);
  const scroller = useRef<HTMLDivElement>(null);
  const content = useRef<HTMLDivElement>(null);
  const readThisFlow = useCallback(
    (start: number, count: number) => readFlow(channel, start, count),
    [readFlow, channel],
  );
  const state = useListState(readThisFlow, redraw);
  followTheChange(state, change);
  followTheTotal(state);
  const range = rangeOnScreen(state);
  if (range.last >= range.first) {
    state.windows.need(range.first, range.last);
  }
  useScrollFollowing(scroller, state, redraw);
  useLayoutAfterDrawing({ scroller, content }, state, redraw);
  const goToTheNewest = useCallback(() => {
    const element = scroller.current;
    state.keepsToTheBottom = true;
    if (element !== null) {
      element.scrollTop = element.scrollHeight;
    }
    redraw();
  }, [state]);
  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      <div ref={scroller} className="min-h-0 flex-1 overflow-auto">
        <EmptyWhenNothingIsWritten state={state} boardUrl={boardUrl} />
        <div ref={content} role="feed" aria-busy={false} className="relative">
          {linesBetween(state, range.first, range.last)}
        </div>
      </div>
      <NewBelow count={newBelowOf(state)} onChoose={goToTheNewest} />
    </div>
  );
}
