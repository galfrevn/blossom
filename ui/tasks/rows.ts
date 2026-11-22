import { useRef } from "react";
import type {
  TaskChange,
  TaskNarrowing,
  TaskRow,
  TaskWindow,
} from "../core/contract";
import { Windows } from "../core/windows";

const ROWS_IN_A_BLOCK = 100;
const BLOCKS_KEPT = 10;
const ROWS_DRAWN_BEYOND_THE_SCREEN = 20;
const ROWS_READ_BEYOND_THE_SCREEN = 60;

function keyOf(narrowing: TaskNarrowing): string {
  const title = narrowing.title.trim().toLowerCase();
  return JSON.stringify([narrowing.channel, narrowing.state, title]);
}

function narrowedRows(
  narrowing: TaskNarrowing,
  readTasks: ReadTasks,
  redraw: () => void,
): NarrowedRows {
  const windows = new Windows<TaskRow>(
    (start, count) => readTasks(narrowing, start, count),
    ROWS_IN_A_BLOCK,
    BLOCKS_KEPT,
    redraw,
  );
  return { narrowing, windows };
}

function rangeAround(
  state: TableState,
  total: number | undefined,
  rowsBeyondTheScreen: number,
) {
  const firstOnScreen = Math.floor(state.scrollTop / ROW_HEIGHT);
  const rowsOnScreen = Math.ceil(state.viewportHeight / ROW_HEIGHT);
  const first = Math.max(0, firstOnScreen - rowsBeyondTheScreen);
  const lastWanted = firstOnScreen + rowsOnScreen + rowsBeyondTheScreen;
  if (total === undefined) {
    return { first, last: lastWanted };
  }
  return { first, last: Math.min(total - 1, lastWanted) };
}

export const ROW_HEIGHT = 30;
export const HEADER_HEIGHT = 28;

export type ReadTasks = (
  narrowing: TaskNarrowing,
  start: number,
  count: number,
) => Promise<TaskWindow>;

export type NarrowedRows = {
  narrowing: TaskNarrowing;
  windows: Windows<TaskRow>;
};

export type TableState = {
  narrowingKey: string;
  latest: NarrowedRows;
  lastAnswered: NarrowedRows | null;
  revision: number;
  scrollTop: number;
  viewportHeight: number;
  backToTheTop: boolean;
};

export function useTableState(
  narrowing: TaskNarrowing,
  readTasks: ReadTasks,
  redraw: () => void,
): TableState {
  const stateRef = useRef<TableState | null>(null);
  if (stateRef.current === null) {
    stateRef.current = {
      narrowingKey: keyOf(narrowing),
      latest: narrowedRows(narrowing, readTasks, redraw),
      lastAnswered: null,
      revision: -1,
      scrollTop: 0,
      viewportHeight: 0,
      backToTheTop: false,
    };
  }
  return stateRef.current;
}

export function followTheNarrowing(
  state: TableState,
  narrowing: TaskNarrowing,
  readTasks: ReadTasks,
  redraw: () => void,
) {
  const narrowingKey = keyOf(narrowing);
  if (narrowingKey === state.narrowingKey) {
    return;
  }
  if (state.latest.windows.total() !== undefined) {
    state.lastAnswered = state.latest;
  }
  state.latest = narrowedRows(narrowing, readTasks, redraw);
  state.narrowingKey = narrowingKey;
  state.scrollTop = 0;
  state.backToTheTop = true;
}

export function followTheChange(
  state: TableState,
  change: TaskChange | undefined,
) {
  if (change === undefined || change.revision === state.revision) {
    return;
  }
  state.revision = change.revision;
  state.latest.windows.renew();
}

export function shownRows(state: TableState): NarrowedRows {
  const latestHasAnswered = state.latest.windows.total() !== undefined;
  if (latestHasAnswered || state.lastAnswered === null) {
    state.lastAnswered = null;
    return state.latest;
  }
  return state.lastAnswered;
}

export function rangeOnScreen(state: TableState, total: number | undefined) {
  return rangeAround(state, total, ROWS_DRAWN_BEYOND_THE_SCREEN);
}

export function rangeToRead(state: TableState, total: number | undefined) {
  return rangeAround(state, total, ROWS_READ_BEYOND_THE_SCREEN);
}

export function narrowsAnything(narrowing: TaskNarrowing): boolean {
  const narrowsByTitle = narrowing.title.trim() !== "";
  return (
    narrowing.channel !== null || narrowing.state !== null || narrowsByTitle
  );
}
