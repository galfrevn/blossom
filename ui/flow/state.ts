import { useRef } from "react";
import type { FlowChange, FlowWindow } from "../core/contract";
import { RowHeights } from "./heights";
import { FlowWindows } from "./windows";

export const ESTIMATED_ROW_HEIGHT = 46;
export const ROWS_IN_A_BLOCK = 100;
export const BLOCKS_KEPT = 30;
export const PIXELS_DRAWN_BEYOND_THE_SCREEN = 600;
export const PIXELS_THAT_STILL_COUNT_AS_THE_BOTTOM = 2;

export type ListState = {
  heights: RowHeights;
  windows: FlowWindows;
  revision: number;
  scrollTop: number;
  viewportHeight: number;
  keepsToTheBottom: boolean;
  totalWhenItLeftTheBottom: number;
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
  state.revision = change.revision;
  if (change.total !== state.heights.count) {
    state.heights = state.heights.resized(change.total);
  }
  state.heights.forgetMeasurements();
  state.windows.renew();
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

export function useListState(
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
      totalWhenItLeftTheBottom: 0,
    };
  }
  return stateRef.current;
}

export function newBelowOf(state: ListState): number {
  if (state.keepsToTheBottom) {
    return 0;
  }
  return Math.max(0, state.heights.count - state.totalWhenItLeftTheBottom);
}
