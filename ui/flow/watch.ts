export type SizeWatcher = Pick<ResizeObserver, "observe" | "unobserve">;

export class WatchedRows {
  private readonly watcher: SizeWatcher;
  private readonly watched = new Set<Element>();

  constructor(watcher: SizeWatcher) {
    this.watcher = watcher;
  }

  follow(rowsOnScreen: Iterable<Element>): void {
    const onScreen = new Set(rowsOnScreen);
    for (const row of this.watched) {
      if (!onScreen.has(row)) {
        this.watcher.unobserve(row);
        this.watched.delete(row);
      }
    }
    for (const row of onScreen) {
      if (!this.watched.has(row)) {
        this.watcher.observe(row);
        this.watched.add(row);
      }
    }
  }
}
