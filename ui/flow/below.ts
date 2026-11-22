import type { FlowRow } from "../core/contract";

export class NewMessagesBelow {
  private readonly countAfter: (
    createdAt: string,
    id: string,
  ) => Promise<number>;
  private readonly onCounted: () => void;
  private newestSeen: FlowRow | undefined;
  private departures = 0;
  private waiting = 0;

  constructor(
    countAfter: (createdAt: string, id: string) => Promise<number>,
    onCounted: () => void,
  ) {
    this.countAfter = countAfter;
    this.onCounted = onCounted;
  }

  leaveTheBottomAt(newestSeen: FlowRow | undefined): void {
    this.newestSeen = newestSeen;
    this.departures += 1;
    this.waiting = 0;
  }

  recount(): void {
    const { newestSeen, departures } = this;
    if (newestSeen === undefined) {
      return;
    }
    this.countAfter(newestSeen.createdAt, newestSeen.id).then(
      (count) => {
        if (departures !== this.departures) {
          return;
        }
        this.waiting = count;
        this.onCounted();
      },
      (error: unknown) => {
        console.error(
          "blossom: the new messages below could not be counted",
          error,
        );
      },
    );
  }

  count(): number {
    return this.waiting;
  }
}
