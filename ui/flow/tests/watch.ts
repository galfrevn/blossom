import { expect, test } from "vitest";
import { WatchedRows } from "../watch";

function recordingWatcher() {
  const calls: string[] = [];
  return {
    calls,
    observe: (row: Element) => {
      calls.push(`watch ${row.id}`);
    },
    unobserve: (row: Element) => {
      calls.push(`stop ${row.id}`);
    },
  };
}

function rowCalled(id: string): Element {
  const row = document.createElement("div");
  row.id = id;
  return row;
}

test("a row on screen is watched once, however many times the list is laid out", () => {
  const watcher = recordingWatcher();
  const rows = new WatchedRows(watcher);
  const first = rowCalled("first");
  const second = rowCalled("second");

  rows.follow([first, second]);
  rows.follow([first, second]);

  expect(watcher.calls).toEqual(["watch first", "watch second"]);
});

test("a row that leaves the screen stops being watched, so it can be let go", () => {
  const watcher = recordingWatcher();
  const rows = new WatchedRows(watcher);
  const first = rowCalled("first");
  const second = rowCalled("second");
  const third = rowCalled("third");
  rows.follow([first, second]);

  rows.follow([second, third]);

  expect(watcher.calls).toEqual([
    "watch first",
    "watch second",
    "stop first",
    "watch third",
  ]);
});
