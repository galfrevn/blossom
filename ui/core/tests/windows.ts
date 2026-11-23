import { expect, test } from "vitest";
import type { Anchor } from "../places";
import { type Window, Windows } from "../windows";

const BOARD_SIZE = 1000;
const ROWS_IN_A_BLOCK = 100;
const ENOUGH_BLOCKS = 10;
const TWO_BLOCKS = 2;
const FIRST_BLOCK = 0;
const SECOND_BLOCK = ROWS_IN_A_BLOCK;
const THIRD_BLOCK = ROWS_IN_A_BLOCK * TWO_BLOCKS;
const A_FEW_ROWS = 10;
const HALF_A_BLOCK = ROWS_IN_A_BLOCK / TWO_BLOCKS;

type NumberedRow = {
  id: string;
  text: string;
};

function rowNumbered(index: number, text: string): NumberedRow {
  return { id: String(index), text };
}

function boardThatSays(textNow: () => string) {
  const requests: number[] = [];
  const read = (start: number, count: number): Promise<Window<NumberedRow>> => {
    requests.push(start);
    const last = Math.min(BOARD_SIZE, start + count);
    const rows: NumberedRow[] = [];
    for (let index = start; index < last; index += 1) {
      rows.push(rowNumbered(index, textNow()));
    }
    return Promise.resolve({ total: BOARD_SIZE, rows });
  };
  return { read, requests };
}

const settle = () => new Promise((resolve) => setTimeout(resolve, 0));
const nothing = () => undefined;

test("the rows near the screen are read a block at a time, once", async () => {
  const board = boardThatSays(() => "work");
  const windows = new Windows(
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
  const windows = new Windows(
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
  const windows = new Windows(board.read, ROWS_IN_A_BLOCK, TWO_BLOCKS, nothing);

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
  const windows = new Windows(
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

const GROWN_BOARD_SIZE = 1500;
const GROWN_AGAIN_BOARD_SIZE = 1600;

function boardThatGrows() {
  const board = { size: BOARD_SIZE, text: "before" };
  const read = (start: number, count: number): Promise<Window<NumberedRow>> => {
    const last = Math.min(board.size, start + count);
    const rows: NumberedRow[] = [];
    for (let index = start; index < last; index += 1) {
      rows.push(rowNumbered(index, board.text));
    }
    return Promise.resolve({ total: board.size, start, rows });
  };
  return { board, read };
}

test("at the bottom, a change shows the newest rows with the new total at once, and the rows before it until then", async () => {
  const { board, read } = boardThatGrows();
  const windows = new Windows(read, ROWS_IN_A_BLOCK, ENOUGH_BLOCKS, nothing);
  windows.need(BOARD_SIZE - ROWS_IN_A_BLOCK, BOARD_SIZE - 1);
  await settle();

  board.size = GROWN_BOARD_SIZE;
  board.text = "after";
  windows.renewTheEnd(A_FEW_ROWS);
  expect(windows.total()).toBe(BOARD_SIZE);
  expect(windows.rowAt(BOARD_SIZE - 1)?.text).toBe("before");
  await settle();

  expect(windows.total()).toBe(GROWN_BOARD_SIZE);
  expect(windows.rowAt(GROWN_BOARD_SIZE - 1)?.text).toBe("after");
  expect(windows.rowAt(BOARD_SIZE - 1)).toBeUndefined();
});

test("a change while the end is read is read too", async () => {
  const { board, read } = boardThatGrows();
  const windows = new Windows(read, ROWS_IN_A_BLOCK, ENOUGH_BLOCKS, nothing);
  windows.need(BOARD_SIZE - ROWS_IN_A_BLOCK, BOARD_SIZE - 1);
  await settle();

  board.size = GROWN_BOARD_SIZE;
  windows.renewTheEnd(A_FEW_ROWS);
  board.size = GROWN_AGAIN_BOARD_SIZE;
  windows.renewTheEnd(A_FEW_ROWS);
  await settle();

  expect(windows.total()).toBe(GROWN_AGAIN_BOARD_SIZE);
  expect(windows.rowAt(GROWN_AGAIN_BOARD_SIZE - 1)?.id).toBe(
    String(GROWN_AGAIN_BOARD_SIZE - 1),
  );
});

function boardAnsweredByHand() {
  const board = { size: BOARD_SIZE };
  const unanswered: Array<() => void> = [];
  const read = (start: number, count: number): Promise<Window<NumberedRow>> =>
    new Promise((resolve) => {
      const sizeWhenAsked = board.size;
      unanswered.push(() => {
        const last = Math.min(sizeWhenAsked, start + count);
        const rows: NumberedRow[] = [];
        for (let index = start; index < last; index += 1) {
          rows.push(rowNumbered(index, "work"));
        }
        resolve({ total: sizeWhenAsked, rows });
      });
    });
  const answerTheNewest = async () => {
    unanswered.pop()?.();
    await settle();
  };
  return { board, read, answerTheNewest };
}

test("a block asked for before the end was read is not shown once the end is", async () => {
  const { board, read, answerTheNewest } = boardAnsweredByHand();
  const windows = new Windows(read, ROWS_IN_A_BLOCK, ENOUGH_BLOCKS, nothing);
  windows.need(FIRST_BLOCK, A_FEW_ROWS);

  board.size = GROWN_BOARD_SIZE;
  windows.renewTheEnd(A_FEW_ROWS);
  await answerTheNewest();
  await answerTheNewest();
  await answerTheNewest();

  expect(windows.total()).toBe(GROWN_BOARD_SIZE);
  expect(windows.rowAt(FIRST_BLOCK + A_FEW_ROWS)).toBeUndefined();
  expect(windows.rowAt(GROWN_BOARD_SIZE - 1)?.id).toBe(
    String(GROWN_BOARD_SIZE - 1),
  );
});

const LANDED_ABOVE = 150;

function boardWhereOlderRowsLand() {
  const board = { landedAbove: 0 };
  const read = (start: number, count: number): Promise<Window<NumberedRow>> => {
    const total = BOARD_SIZE + board.landedAbove;
    const rows: NumberedRow[] = [];
    for (
      let index = start;
      index < Math.min(total, start + count);
      index += 1
    ) {
      rows.push(rowNumbered(index - board.landedAbove, "work"));
    }
    return Promise.resolve({ total, rows });
  };
  return { board, read };
}

function lineHeld(
  board: { landedAbove: number },
  id: number,
): Anchor<NumberedRow> {
  return {
    find: () => Promise.resolve(id + board.landedAbove),
    isAtOrAfter: (row) => Number(row.id) >= id,
    rowsBefore: A_FEW_ROWS,
    rowsAfter: A_FEW_ROWS,
  };
}

test("away from the end, a change shows the rows around the line held, where it sits now, with the new total at once", async () => {
  const { board, read } = boardWhereOlderRowsLand();
  const windows = new Windows(read, ROWS_IN_A_BLOCK, ENOUGH_BLOCKS, nothing);
  windows.need(SECOND_BLOCK, SECOND_BLOCK + A_FEW_ROWS);
  await settle();
  const found: number[] = [];

  board.landedAbove = LANDED_ABOVE;
  windows.renewAround(lineHeld(board, SECOND_BLOCK), (index) => {
    found.push(index);
  });
  expect(windows.total()).toBe(BOARD_SIZE);
  expect(windows.rowAt(SECOND_BLOCK)?.id).toBe(String(SECOND_BLOCK));
  await settle();

  const whereItSitsNow = SECOND_BLOCK + LANDED_ABOVE;
  expect(found).toEqual([whereItSitsNow]);
  expect(windows.total()).toBe(BOARD_SIZE + LANDED_ABOVE);
  expect(windows.rowAt(whereItSitsNow)?.id).toBe(String(SECOND_BLOCK));
  expect(windows.rowAt(SECOND_BLOCK)).toBeUndefined();
});

test("a change at the end while a line is held is read once the line is shown", async () => {
  const { board, read } = boardWhereOlderRowsLand();
  const windows = new Windows(read, ROWS_IN_A_BLOCK, ENOUGH_BLOCKS, nothing);
  const found: number[] = [];

  board.landedAbove = LANDED_ABOVE;
  windows.renewAround(lineHeld(board, SECOND_BLOCK), (index) => {
    found.push(index);
  });
  windows.renewTheEnd(A_FEW_ROWS);
  expect(windows.isReadingThePlace()).toBe(true);
  await settle();

  const totalNow = BOARD_SIZE + LANDED_ABOVE;
  expect(found).toEqual([SECOND_BLOCK + LANDED_ABOVE]);
  expect(windows.isReadingThePlace()).toBe(false);
  expect(windows.rowAt(totalNow - 1)?.id).toBe(String(BOARD_SIZE - 1));
});
