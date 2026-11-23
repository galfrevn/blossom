import type { FlowRow } from "../core/contract";
import type { Anchor } from "../core/places";
import type { RowHeights } from "./heights";

function comesAtOrAfter(row: FlowRow, line: FlowRow): boolean {
  if (row.createdAt !== line.createdAt) {
    return row.createdAt > line.createdAt;
  }
  return row.id >= line.id;
}

export type Move = {
  from: number;
  to: number;
};

export type RowsAround = {
  before: number;
  after: number;
};

export function lineStartingAt(heights: RowHeights, scrollTop: number): number {
  const index = heights.rowAt(scrollTop);
  const startsAboveTheScreen = heights.offsetOf(index) < scrollTop;
  if (startsAboveTheScreen && index + 1 < heights.count) {
    return index + 1;
  }
  return index;
}

export function anchorAt(
  line: FlowRow,
  countBefore: (createdAt: string, id: string) => Promise<number>,
  rowsAround: RowsAround,
): Anchor<FlowRow> {
  return {
    find: () => countBefore(line.createdAt, line.id),
    isAtOrAfter: (row) => comesAtOrAfter(row, line),
    rowsBefore: rowsAround.before,
    rowsAfter: rowsAround.after,
  };
}

export function scrollTopAfter(
  move: Move,
  before: RowHeights,
  after: RowHeights,
  scrollTop: number,
): number {
  const heightOnScreen = before.offsetOf(move.from) - scrollTop;
  return after.offsetOf(move.to) - heightOnScreen;
}
