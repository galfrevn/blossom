type Block<Row> = {
  rows: Row[];
  generation: number;
};

type ReadingOfTheEnd = "notReading" | "reading" | "readingAndRenewedSince";

export type Window<Row> = {
  total: number;
  rows: Row[];
};

export class Windows<Row> {
  private readonly read: (start: number, count: number) => Promise<Window<Row>>;
  private readonly blockSize: number;
  private readonly blocksKept: number;
  private readonly onArrival: () => void;
  private readonly blocks = new Map<number, Block<Row>>();
  private readonly requested = new Map<number, number>();
  private generation = 0;
  private knownTotal: number | undefined;
  private rowsAtTheEnd = 1;
  private readingOfTheEnd: ReadingOfTheEnd = "notReading";
  private generationWhenTheEndWasRead = 0;

  constructor(
    read: (start: number, count: number) => Promise<Window<Row>>,
    blockSize: number,
    blocksKept: number,
    onArrival: () => void,
  ) {
    this.read = read;
    this.blockSize = blockSize;
    this.blocksKept = blocksKept;
    this.onArrival = onArrival;
  }

  rowAt(index: number): Row | undefined {
    const block = Math.floor(index / this.blockSize);
    return this.blocks.get(block)?.rows[index - block * this.blockSize];
  }

  need(first: number, last: number): void {
    if (this.readingOfTheEnd !== "notReading") {
      return;
    }
    const firstBlock = Math.floor(first / this.blockSize);
    const lastBlock = Math.floor(last / this.blockSize);
    for (let block = firstBlock; block <= lastBlock; block += 1) {
      const kept = this.blocks.get(block);
      if (kept !== undefined) {
        this.blocks.delete(block);
        this.blocks.set(block, kept);
      }
      const isFresh = kept?.generation === this.generation;
      const isAsked = this.requested.get(block) === this.generation;
      if (!(isFresh || isAsked)) {
        this.request(block);
      }
    }
  }

  renew(): void {
    this.generation += 1;
  }

  renewTheEnd(rowsAtTheEnd: number): void {
    this.generation += 1;
    this.rowsAtTheEnd = Math.max(1, rowsAtTheEnd);
    if (this.readingOfTheEnd !== "notReading") {
      this.readingOfTheEnd = "readingAndRenewedSince";
      return;
    }
    this.readTheEnd();
  }

  total(): number | undefined {
    return this.knownTotal;
  }

  private request(block: number): void {
    const { generation } = this;
    this.requested.set(block, generation);
    this.read(block * this.blockSize, this.blockSize).then(
      (window) => {
        this.requested.delete(block);
        if (generation < this.generationWhenTheEndWasRead) {
          return;
        }
        this.knownTotal = window.total;
        this.blocks.delete(block);
        this.blocks.set(block, { rows: window.rows, generation });
        this.letGoOfTheOldest();
        this.onArrival();
      },
      (error: unknown) => {
        this.requested.delete(block);
        console.error("blossom: a window of rows could not be read", error);
      },
    );
  }

  private readTheEnd(): void {
    this.readingOfTheEnd = "reading";
    this.generationWhenTheEndWasRead = this.generation;
    this.readTheEndOf(this.knownTotal ?? 0);
  }

  private readTheEndOf(total: number): void {
    const blocks = this.blocksAtTheEndOf(total);
    const reads = blocks.map((block) =>
      this.read(block * this.blockSize, this.blockSize),
    );
    Promise.all(reads).then(
      (windows) => {
        const totalNow = windows.at(-1)?.total ?? total;
        const everyReadAgrees = windows.every(
          (window) => window.total === totalNow,
        );
        const endNow = this.blocksAtTheEndOf(totalNow);
        const theEndIsWhatWasRead =
          endNow.length === blocks.length &&
          endNow.every((block, index) => block === blocks[index]);
        if (!(everyReadAgrees && theEndIsWhatWasRead)) {
          this.readTheEndOf(totalNow);
          return;
        }
        this.showTheEnd(blocks, windows, totalNow);
      },
      (error: unknown) => {
        this.readingOfTheEnd = "notReading";
        console.error("blossom: the end of the list could not be read", error);
      },
    );
  }

  private showTheEnd(
    blocks: number[],
    windows: Window<Row>[],
    total: number,
  ): void {
    this.blocks.clear();
    this.knownTotal = total;
    const generation = this.generationWhenTheEndWasRead;
    for (const [index, block] of blocks.entries()) {
      this.blocks.set(block, { rows: windows[index]?.rows ?? [], generation });
    }
    if (this.readingOfTheEnd === "readingAndRenewedSince") {
      this.readTheEnd();
    } else {
      this.readingOfTheEnd = "notReading";
    }
    this.onArrival();
  }

  private blocksAtTheEndOf(total: number): number[] {
    const firstRow = Math.max(0, total - this.rowsAtTheEnd);
    const lastRow = Math.max(0, total - 1);
    const blocks: number[] = [];
    const lastBlock = Math.floor(lastRow / this.blockSize);
    for (
      let block = Math.floor(firstRow / this.blockSize);
      block <= lastBlock;
      block += 1
    ) {
      blocks.push(block);
    }
    return blocks;
  }

  private letGoOfTheOldest(): void {
    for (const block of this.blocks.keys()) {
      if (this.blocks.size <= this.blocksKept) {
        return;
      }
      this.blocks.delete(block);
    }
  }
}
