import type { Window } from "./windows";

function blocksBetween(
  firstRow: number,
  lastRow: number,
  blockSize: number,
): number[] {
  const blocks: number[] = [];
  const lastBlock = Math.floor(lastRow / blockSize);
  for (
    let block = Math.floor(firstRow / blockSize);
    block <= lastBlock;
    block += 1
  ) {
    blocks.push(block);
  }
  return blocks;
}

function blocksAtTheEndOf(
  total: number,
  rowsAtTheEnd: number,
  blockSize: number,
): number[] {
  const firstRow = Math.max(0, total - rowsAtTheEnd);
  const lastRow = Math.max(0, total - 1);
  return blocksBetween(firstRow, lastRow, blockSize);
}

function readBlocks<Row>(
  blocks: number[],
  reader: BlockReader<Row>,
): Promise<PlaceRead<Row>> {
  return Promise.all(blocks.map((block) => reader.read(block))).then(
    (windows) => ({ blocks, windows, total: windows.at(-1)?.total ?? 0 }),
  );
}

function everyReadAgrees<Row>(read: PlaceRead<Row>): boolean {
  return read.windows.every((window) => window.total === read.total);
}

function areTheSame(blocks: number[], others: number[]): boolean {
  return (
    blocks.length === others.length &&
    blocks.every((block, index) => block === others[index])
  );
}

function indexOfTheAnchorIn<Row>(
  read: PlaceRead<Row>,
  anchor: Anchor<Row>,
  blockSize: number,
): number | undefined {
  if (!everyReadAgrees(read)) {
    return undefined;
  }
  const firstRow = (read.blocks[0] ?? 0) * blockSize;
  const rows = read.windows.flatMap((window) => window.rows);
  const offset = rows.findIndex((row) => anchor.isAtOrAfter(row));
  if (offset === -1) {
    const theEndWasRead = firstRow + rows.length >= read.total;
    if (theEndWasRead) {
      return read.total;
    }
    return undefined;
  }
  const theRowBeforeWasRead = offset > 0 || firstRow === 0;
  if (theRowBeforeWasRead) {
    return firstRow + offset;
  }
  return undefined;
}

export type Anchor<Row> = {
  find: () => Promise<number>;
  isAtOrAfter: (row: Row) => boolean;
  rowsBefore: number;
  rowsAfter: number;
};

export type BlockReader<Row> = {
  read: (block: number) => Promise<Window<Row>>;
  size: number;
};

export type PlaceRead<Row> = {
  blocks: number[];
  windows: Window<Row>[];
  total: number;
};

export type AnchorRead<Row> = {
  read: PlaceRead<Row>;
  index: number;
};

export function readTheEnd<Row>(
  total: number,
  rowsAtTheEnd: number,
  reader: BlockReader<Row>,
): Promise<PlaceRead<Row>> {
  const atTheEnd = blocksAtTheEndOf(total, rowsAtTheEnd, reader.size);
  return readBlocks(atTheEnd, reader).then((read) => {
    const endNow = blocksAtTheEndOf(read.total, rowsAtTheEnd, reader.size);
    if (everyReadAgrees(read) && areTheSame(atTheEnd, endNow)) {
      return read;
    }
    return readTheEnd(read.total, rowsAtTheEnd, reader);
  });
}

export function readAround<Row>(
  anchor: Anchor<Row>,
  reader: BlockReader<Row>,
): Promise<AnchorRead<Row>> {
  return anchor
    .find()
    .then((index) => {
      const firstRow = Math.max(0, index - anchor.rowsBefore - 1);
      const lastRow = Math.max(firstRow, index + anchor.rowsAfter);
      return readBlocks(blocksBetween(firstRow, lastRow, reader.size), reader);
    })
    .then((read) => {
      const index = indexOfTheAnchorIn(read, anchor, reader.size);
      if (index === undefined) {
        return readAround(anchor, reader);
      }
      return { read, index };
    });
}
