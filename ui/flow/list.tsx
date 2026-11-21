import { memo, useEffect, useLayoutEffect, useReducer, useRef } from "react";
import type { FlowChange, FlowRow, FlowWindow } from "../core/contract";
import { RowHeights } from "./heights";
import { FlowLine } from "./line";
import { FlowWindows } from "./windows";

const ESTIMATED_ROW_HEIGHT = 46;
const ROWS_IN_A_BLOCK = 100;
const BLOCKS_KEPT = 30;
const PIXELS_DRAWN_BEYOND_THE_SCREEN = 600;
const PIXELS_THAT_STILL_COUNT_AS_THE_BOTTOM = 2;

type ListState = {
  heights: RowHeights;
  windows: FlowWindows;
  revision: number;
  scrollTop: number;
  viewportHeight: number;
  keepsToTheBottom: boolean;
};

function isAtTheBottom(scroller: HTMLElement): boolean {
  const distance =
    scroller.scrollHeight - scroller.clientHeight - scroller.scrollTop;
  return distance <= PIXELS_THAT_STILL_COUNT_AS_THE_BOTTOM;
}

function followTheChange(state: ListState, change: FlowChange | undefined) {
  if (change === undefined || change.revision === state.revision) {
    return;
  }
  state.revision = change.revision;
  if (change.total !== state.heights.count) {
    state.heights = state.heights.resized(change.total);
  }
  state.heights.forgetMeasurements();
  state.windows.renew();
}

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

function heightStillToMeasure(
  element: HTMLElement,
  state: ListState,
  index: number,
): number {
  if (state.heights.isMeasured(index)) {
    return 0;
  }
  return element.offsetHeight;
}

function measure(scroller: HTMLElement, state: ListState) {
  let shiftAbove = 0;
  let changed = false;
  for (const element of scroller.querySelectorAll<HTMLElement>(
    "[data-loaded]",
  )) {
    const index = Number(element.dataset.index);
    const height = heightStillToMeasure(element, state, index);
    if (height > 0) {
      const top = state.heights.offsetOf(index);
      const change = state.heights.set(index, height);
      if (change !== 0) {
        changed = true;
        if (top < state.scrollTop) {
          shiftAbove += change;
        }
      }
    }
  }
  for (const element of scroller.querySelectorAll<HTMLElement>(
    "[data-placeholder]",
  )) {
    const index = Number(element.dataset.index);
    element.style.height = `${state.heights.heightOf(index)}px`;
  }
  return { shiftAbove, changed };
}

function place(scroller: HTMLElement, state: ListState) {
  for (const element of scroller.querySelectorAll<HTMLElement>(
    "[data-index]",
  )) {
    const top = state.heights.offsetOf(Number(element.dataset.index));
    element.style.transform = `translateY(${top}px)`;
  }
}

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

function rangeOnScreen(state: ListState) {
  if (state.heights.count === 0) {
    return { first: 0, last: -1 };
  }
  const first = state.heights.rowAt(
    Math.max(0, state.scrollTop - PIXELS_DRAWN_BEYOND_THE_SCREEN),
  );
  const last = state.heights.rowAt(
    state.scrollTop + state.viewportHeight + PIXELS_DRAWN_BEYOND_THE_SCREEN,
  );
  return { first, last };
}

function useListState(
  readFlow: (start: number, count: number) => Promise<FlowWindow>,
  redraw: () => void,
): ListState {
  const stateRef = useRef<ListState | null>(null);
  if (stateRef.current === null) {
    stateRef.current = {
      heights: new RowHeights(0, ESTIMATED_ROW_HEIGHT),
      windows: new FlowWindows(readFlow, ROWS_IN_A_BLOCK, BLOCKS_KEPT, redraw),
      revision: -1,
      scrollTop: 0,
      viewportHeight: 0,
      keepsToTheBottom: true,
    };
  }
  return stateRef.current;
}

function useScrollFollowing(
  scrollerRef: React.RefObject<HTMLDivElement | null>,
  state: ListState,
  redraw: () => void,
) {
  useEffect(() => {
    const scroller = scrollerRef.current;
    if (scroller === null) {
      return;
    }
    let frame = 0;
    const onScroll = () => {
      if (frame !== 0) {
        return;
      }
      frame = requestAnimationFrame(() => {
        frame = 0;
        state.scrollTop = scroller.scrollTop;
        state.viewportHeight = scroller.clientHeight;
        state.keepsToTheBottom = isAtTheBottom(scroller);
        redraw();
      });
    };
    state.viewportHeight = scroller.clientHeight;
    scroller.addEventListener("scroll", onScroll, { passive: true });
    let width = scroller.clientWidth;
    const resizing = new ResizeObserver(() => {
      if (scroller.clientWidth === width) {
        return;
      }
      width = scroller.clientWidth;
      state.heights.forgetMeasurements();
      redraw();
    });
    resizing.observe(scroller);
    return () => {
      cancelAnimationFrame(frame);
      resizing.disconnect();
      scroller.removeEventListener("scroll", onScroll);
    };
  }, [scrollerRef, state, redraw]);
}

type ListElements = {
  scroller: React.RefObject<HTMLDivElement | null>;
  content: React.RefObject<HTMLDivElement | null>;
};

function useLayoutAfterDrawing(
  elements: ListElements,
  state: ListState,
  redraw: () => void,
) {
  useLayoutEffect(() => {
    const scroller = elements.scroller.current;
    const content = elements.content.current;
    if (scroller === null || content === null) {
      return;
    }
    const { shiftAbove, changed } = measure(scroller, state);
    const { heights } = state;
    content.style.height = `${heights.total()}px`;
    place(scroller, state);
    if (state.keepsToTheBottom && heights.count > 0) {
      scroller.scrollTop = scroller.scrollHeight;
    } else if (shiftAbove !== 0) {
      scroller.scrollTop += shiftAbove;
    }
    const scrollMoved = scroller.scrollTop !== state.scrollTop;
    state.scrollTop = scroller.scrollTop;
    state.viewportHeight = scroller.clientHeight;
    if (changed || scrollMoved) {
      redraw();
    }
  });
}

export function FlowList({
  change,
  readFlow,
}: {
  change: FlowChange | undefined;
  readFlow: (start: number, count: number) => Promise<FlowWindow>;
}) {
  const [, redraw] = useReducer((count: number) => count + 1, 0);
  const scroller = useRef<HTMLDivElement>(null);
  const content = useRef<HTMLDivElement>(null);
  const state = useListState(readFlow, redraw);
  followTheChange(state, change);
  const range = rangeOnScreen(state);
  if (range.last >= range.first) {
    state.windows.need(range.first, range.last);
  }
  useScrollFollowing(scroller, state, redraw);
  useLayoutAfterDrawing({ scroller, content }, state, redraw);
  return (
    <div ref={scroller} className="min-h-0 flex-1 overflow-auto">
      <div ref={content} role="feed" aria-busy={false} className="relative">
        {linesBetween(state, range.first, range.last)}
      </div>
    </div>
  );
}
