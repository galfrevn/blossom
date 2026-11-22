type Block<Row> = {
  rows: Row[];
  generation: number;
};

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

  total(): number | undefined {
    return this.knownTotal;
  }

  private request(block: number): void {
    const { generation } = this;
    this.requested.set(block, generation);
    this.read(block * this.blockSize, this.blockSize).then(
      (window) => {
        this.requested.delete(block);
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

  private letGoOfTheOldest(): void {
    for (const block of this.blocks.keys()) {
      if (this.blocks.size <= this.blocksKept) {
        return;
      }
      this.blocks.delete(block);
    }
  }
}
