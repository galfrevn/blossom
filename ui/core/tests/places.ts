import { expect, test } from "vitest";
import { type Anchor, readAround } from "../places";
import type { Window } from "../windows";

const BOARD_SIZE = 1000;
const ROWS_IN_A_BLOCK = 100;
const A_FEW_ROWS = 10;
const HELD_LINE = 100;
const LANDED_ABOVE = 100;
const THE_BLOCK_BEFORE_IT = 1;
const ITS_BLOCK = 2;
const ASKED_ONCE = 1;
const ASKED_TWICE = 2;

type NumberedRow = {
  id: number;
};

type Board = {
  landedAbove: number;
  finds: number;
};

function boardWhereOlderRowsLand() {
  const board: Board = { landedAbove: 0, finds: 0 };
  const read = (block: number): Promise<Window<NumberedRow>> => {
    const total = BOARD_SIZE + board.landedAbove;
    const first = block * ROWS_IN_A_BLOCK;
    const last = Math.min(total, first + ROWS_IN_A_BLOCK);
    const rows: NumberedRow[] = [];
    for (let index = first; index < last; index += 1) {
      rows.push({ id: index - board.landedAbove });
    }
    return Promise.resolve({ total, rows });
  };
  return { board, blocks: { read, size: ROWS_IN_A_BLOCK } };
}

function lineHeld(board: Board, id: number): Anchor<NumberedRow> {
  return {
    find: () => {
      board.finds += 1;
      return Promise.resolve(id + board.landedAbove);
    },
    isAtOrAfter: (row) => row.id >= id,
    rowsBefore: A_FEW_ROWS,
    rowsAfter: A_FEW_ROWS,
  };
}

function movingAfterTheFirstAnswer(
  board: Board,
  held: Anchor<NumberedRow>,
  move: () => void,
): Anchor<NumberedRow> {
  return {
    ...held,
    find: () => {
      const answer = held.find();
      if (board.finds === ASKED_ONCE) {
        move();
      }
      return answer;
    },
  };
}

test("the rows around a line held are read where it sits now, with the row before it", async () => {
  const { board, blocks } = boardWhereOlderRowsLand();
  board.landedAbove = LANDED_ABOVE;

  const { read, index } = await readAround(lineHeld(board, HELD_LINE), blocks);

  expect(index).toBe(HELD_LINE + LANDED_ABOVE);
  expect(read.blocks).toEqual([THE_BLOCK_BEFORE_IT, ITS_BLOCK]);
  expect(read.total).toBe(BOARD_SIZE + LANDED_ABOVE);
  expect(board.finds).toBe(ASKED_ONCE);
});

test("a line held that moved a row while its rows were read is found among them", async () => {
  const { board, blocks } = boardWhereOlderRowsLand();
  const held = movingAfterTheFirstAnswer(
    board,
    lineHeld(board, HELD_LINE),
    () => {
      board.landedAbove += 1;
    },
  );

  const { index } = await readAround(held, blocks);

  expect(index).toBe(HELD_LINE + 1);
  expect(board.finds).toBe(ASKED_ONCE);
});

test("a line held that moved beyond the rows read is looked for again", async () => {
  const { board, blocks } = boardWhereOlderRowsLand();
  const held = movingAfterTheFirstAnswer(
    board,
    lineHeld(board, HELD_LINE),
    () => {
      board.landedAbove = LANDED_ABOVE;
    },
  );

  const { index } = await readAround(held, blocks);

  expect(index).toBe(HELD_LINE + LANDED_ABOVE);
  expect(board.finds).toBe(ASKED_TWICE);
});

test("a line held after every row sits at the end, once the end is read", async () => {
  const { board, blocks } = boardWhereOlderRowsLand();

  const { index } = await readAround(lineHeld(board, BOARD_SIZE), blocks);

  expect(index).toBe(BOARD_SIZE);
});
