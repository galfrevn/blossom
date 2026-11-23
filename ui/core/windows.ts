import {
  type Anchor,
  type BlockReader,
  type PlaceRead,
  readAround,
  readTheEnd,
} from "./places";

type Block<Row> = {
  rows: Row[];
  generation: number;
};

type ReadingOfThePlace = "notReading" | "reading" | "readingAndRenewedSince";

type Place<Row> =
  | { rowsAtTheEnd: number }
  | { anchor: Anchor<Row>; onFound: (index: number) => void };

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
  private placeToRead: Place<Row> = { rowsAtTheEnd: 1 };
  private readingOfThePlace: ReadingOfThePlace = "notReading";
  private generationWhenThePlaceWasRead = 0;

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
    if (this.isReadingThePlace()) {
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
    this.renewThePlace({ rowsAtTheEnd: Math.max(1, rowsAtTheEnd) });
  }

  renewAround(anchor: Anchor<Row>, onFound: (index: number) => void): void {
    this.renewThePlace({ anchor, onFound });
  }

  isReadingThePlace(): boolean {
    return this.readingOfThePlace !== "notReading";
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
        if (generation < this.generationWhenThePlaceWasRead) {
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

  private renewThePlace(place: Place<Row>): void {
    this.generation += 1;
    this.placeToRead = place;
    if (this.isReadingThePlace()) {
      this.readingOfThePlace = "readingAndRenewedSince";
      return;
    }
    this.readThePlace();
  }

  private readThePlace(): void {
    this.readingOfThePlace = "reading";
    this.generationWhenThePlaceWasRead = this.generation;
    const place = this.placeToRead;
    const stop = (error: unknown) => {
      this.stopReadingThePlace(error);
    };
    if ("anchor" in place) {
      readAround(place.anchor, this.blockReader()).then(({ read, index }) => {
        place.onFound(index);
        this.showThePlace(read);
      }, stop);
      return;
    }
    readTheEnd(
      this.knownTotal ?? 0,
      place.rowsAtTheEnd,
      this.blockReader(),
    ).then((read) => {
      this.showThePlace(read);
    }, stop);
  }

  private blockReader(): BlockReader<Row> {
    return {
      read: (block) => this.read(block * this.blockSize, this.blockSize),
      size: this.blockSize,
    };
  }

  private showThePlace(read: PlaceRead<Row>): void {
    this.blocks.clear();
    this.knownTotal = read.total;
    const generation = this.generationWhenThePlaceWasRead;
    for (const [index, block] of read.blocks.entries()) {
      const rows = read.windows[index]?.rows ?? [];
      this.blocks.set(block, { rows, generation });
    }
    if (this.readingOfThePlace === "readingAndRenewedSince") {
      this.readThePlace();
    } else {
      this.readingOfThePlace = "notReading";
    }
    this.onArrival();
  }

  private stopReadingThePlace(error: unknown): void {
    this.readingOfThePlace = "notReading";
    console.error("blossom: the rows around a place could not be read", error);
    this.onArrival();
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
