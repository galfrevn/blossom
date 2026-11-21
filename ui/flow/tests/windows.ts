import { expect, test } from "vitest";
import type { FlowRow, FlowWindow } from "../../core/contract";
import { FlowWindows } from "../windows";

const BOARD_SIZE = 1000;
const ROWS_IN_A_BLOCK = 100;
const ENOUGH_BLOCKS = 10;
const TWO_BLOCKS = 2;
const FIRST_BLOCK = 0;
const SECOND_BLOCK = ROWS_IN_A_BLOCK;
const THIRD_BLOCK = ROWS_IN_A_BLOCK * TWO_BLOCKS;
const A_FEW_ROWS = 10;
const HALF_A_BLOCK = ROWS_IN_A_BLOCK / TWO_BLOCKS;

function rowNumbered(index: number, text: string): FlowRow {
  return {
    id: String(index),
    channel: "fleet/www",
    author: "claude",
    onBehalfOf: "ada",
    createdAt: "2026-11-21T10:00:00.000Z",
    text,
    edited: false,
    deleted: false,
    firstOfItsMinute: false,
    fold: null,
  };
}

function boardThatSays(textNow: () => string) {
  const requests: number[] = [];
  const read = (start: number, count: number): Promise<FlowWindow> => {
    requests.push(start);
    const last = Math.min(BOARD_SIZE, start + count);
    const rows: FlowRow[] = [];
    for (let index = start; index < last; index += 1) {
      rows.push(rowNumbered(index, textNow()));
    }
    return Promise.resolve({ total: BOARD_SIZE, start, rows });
  };
  return { read, requests };
}

const settle = () => new Promise((resolve) => setTimeout(resolve, 0));
const nothing = () => undefined;

test("the rows near the screen are read a block at a time, once", async () => {
  const board = boardThatSays(() => "work");
  const windows = new FlowWindows(
    board.read,
    ROWS_IN_A_BLOCK,
    ENOUGH_BLOCKS,
    nothing,
  );

  windows.need(FIRST_BLOCK, SECOND_BLOCK + HALF_A_BLOCK);
  windows.need(FIRST_BLOCK + A_FEW_ROWS, SECOND_BLOCK + A_FEW_ROWS);
  await settle();

  const lastRead = SECOND_BLOCK + HALF_A_BLOCK;
  expect(board.requests).toEqual([FIRST_BLOCK, SECOND_BLOCK]);
  expect(windows.rowAt(lastRead)?.id).toBe(String(lastRead));
  expect(windows.rowAt(THIRD_BLOCK + A_FEW_ROWS)).toBeUndefined();
});

test("it tells the list when a block arrives", async () => {
  const board = boardThatSays(() => "work");
  let arrivals = 0;
  const windows = new FlowWindows(
    board.read,
    ROWS_IN_A_BLOCK,
    ENOUGH_BLOCKS,
    () => {
      arrivals += 1;
    },
  );

  windows.need(FIRST_BLOCK, A_FEW_ROWS);
  await settle();

  expect(arrivals).toBe(1);
});

test("only the blocks needed most recently are kept", async () => {
  const board = boardThatSays(() => "work");
  const windows = new FlowWindows(
    board.read,
    ROWS_IN_A_BLOCK,
    TWO_BLOCKS,
    nothing,
  );

  windows.need(FIRST_BLOCK, FIRST_BLOCK + A_FEW_ROWS);
  windows.need(SECOND_BLOCK, SECOND_BLOCK + A_FEW_ROWS);
  windows.need(THIRD_BLOCK, THIRD_BLOCK + A_FEW_ROWS);
  await settle();

  expect(windows.rowAt(FIRST_BLOCK + A_FEW_ROWS)).toBeUndefined();
  expect(windows.rowAt(SECOND_BLOCK + A_FEW_ROWS)?.id).toBe(
    String(SECOND_BLOCK + A_FEW_ROWS),
  );
  expect(windows.rowAt(THIRD_BLOCK + A_FEW_ROWS)?.id).toBe(
    String(THIRD_BLOCK + A_FEW_ROWS),
  );
});

test("after a change on the board, the blocks in view are read again and shown until then", async () => {
  let textNow = "before";
  const board = boardThatSays(() => textNow);
  const windows = new FlowWindows(
    board.read,
    ROWS_IN_A_BLOCK,
    ENOUGH_BLOCKS,
    nothing,
  );
  windows.need(FIRST_BLOCK, A_FEW_ROWS);
  await settle();

  textNow = "after";
  windows.renew();
  expect(windows.rowAt(A_FEW_ROWS)?.text).toBe("before");
  windows.need(FIRST_BLOCK, A_FEW_ROWS);
  await settle();

  expect(board.requests).toEqual([FIRST_BLOCK, FIRST_BLOCK]);
  expect(windows.rowAt(A_FEW_ROWS)?.text).toBe("after");
});
