import { useRef } from "react";
import type { FlowChange, FlowRow, FlowWindow } from "../core/contract";
import { Windows } from "../core/windows";
import { NewMessagesBelow } from "./below";
import { RowHeights } from "./heights";
import { anchorAt, lineStartingAt, type Move, scrollTopAfter } from "./place";
import type { WatchedRows } from "./watch";

function takeTheChange(state: ListState, change: FlowChange) {
  state.revision = change.revision;
  state.heights.forgetMeasurements();
}

function holdTheLineOnScreen(state: ListState) {
  const range = rangeOnScreen(state);
  const index = lineStartingAt(state.heights, state.scrollTop);
  const line = state.windows.rowAt(index);
  if (line === undefined) {
    state.windows.renew();
    return;
  }
  const rowsAround = { before: index - range.first, after: range.last - index };
  const anchor = anchorAt(line, state.countBefore, rowsAround);
  state.windows.renewAround(anchor, (found) => {
    state.move = { from: index, to: found };
  });
}

export const ESTIMATED_ROW_HEIGHT = 46;
export const ROWS_IN_A_BLOCK = 100;
export const BLOCKS_KEPT = 30;
export const PIXELS_DRAWN_BEYOND_THE_SCREEN = 600;
export const PIXELS_THAT_STILL_COUNT_AS_THE_BOTTOM = 2;

export type ListState = {
  heights: RowHeights;
  windows: Windows<FlowRow>;
  revision: number;
  scrollTop: number;
  viewportHeight: number;
  keepsToTheBottom: boolean;
  newBelow: NewMessagesBelow;
  watchedRows: WatchedRows | null;
  countBefore: (createdAt: string, id: string) => Promise<number>;
  move: Move | undefined;
  wantedScrollTop: number | undefined;
};

export type FlowReads = {
  readFlow: (start: number, count: number) => Promise<FlowWindow>;
  countFlowBefore: (createdAt: string, id: string) => Promise<number>;
  countFlowAfter: (createdAt: string, id: string) => Promise<number>;
};

export function isAtTheBottom(scroller: HTMLElement): boolean {
  const distance =
    scroller.scrollHeight - scroller.clientHeight - scroller.scrollTop;
  return distance <= PIXELS_THAT_STILL_COUNT_AS_THE_BOTTOM;
}

export function followTheChange(
  state: ListState,
  change: FlowChange | undefined,
) {
  if (change === undefined || change.revision === state.revision) {
    return;
  }
  if (state.keepsToTheBottom) {
    takeTheChange(state, change);
    const range = rangeOnScreen(state);
    state.windows.renewTheEnd(range.last - range.first + 1);
    return;
  }
  if (state.windows.isReadingThePlace()) {
    return;
  }
  takeTheChange(state, change);
  holdTheLineOnScreen(state);
  state.newBelow.recount();
}

export function followTheMove(state: ListState) {
  const { move } = state;
  if (move === undefined) {
    return;
  }
  state.move = undefined;
  const before = state.heights;
  state.heights = before.resized(state.windows.total() ?? before.count);
  if (state.keepsToTheBottom) {
    return;
  }
  state.scrollTop = scrollTopAfter(
    move,
    before,
    state.heights,
    state.scrollTop,
  );
  state.wantedScrollTop = state.scrollTop;
}

export function followTheTotal(state: ListState) {
  const total = state.windows.total();
  if (total === undefined) {
    state.windows.need(0, 0);
    return;
  }
  if (total !== state.heights.count) {
    state.heights = state.heights.resized(total);
  }
}

export function rangeOnScreen(state: ListState) {
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

export function useListState(reads: FlowReads, redraw: () => void): ListState {
  const stateRef = useRef<ListState | null>(null);
  if (stateRef.current === null) {
    stateRef.current = {
      heights: new RowHeights(0, ESTIMATED_ROW_HEIGHT),
      windows: new Windows(
        reads.readFlow,
        ROWS_IN_A_BLOCK,
        BLOCKS_KEPT,
        redraw,
      ),
      revision: -1,
      scrollTop: 0,
      viewportHeight: 0,
      keepsToTheBottom: true,
      newBelow: new NewMessagesBelow(reads.countFlowAfter, redraw),
      watchedRows: null,
      countBefore: reads.countFlowBefore,
      move: undefined,
      wantedScrollTop: undefined,
    };
  }
  return stateRef.current;
}

export function newBelowOf(state: ListState): number {
  if (state.keepsToTheBottom) {
    return 0;
  }
  return state.newBelow.count();
}
