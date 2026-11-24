import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import { createClock } from "../../core/clock";
import type {
  TaskChange,
  TaskNarrowing,
  TaskRow,
  TaskWindow,
} from "../../core/contract";
import { createFakeCore } from "../../core/fake";
import { dictionary } from "../../dictionary";
import { HEADER_HEIGHT, type ReadTasks, ROW_HEIGHT } from "../rows";
import { TaskTable } from "../table";

afterEach(cleanup);

const ONE_SECOND = 1000;
const A_THOUSAND_TASKS = 1000;
const FEWER_THAN_A_TENTH = A_THOUSAND_TASKS / 10;
const SCREEN = 600;
const DEEP_ROW = 500;
const ROWS_PAST_THE_SCREEN = 25;
const A_ROW_HALF_A_SCREEN_INTO_THE_FIRST_WINDOW = 30;
const FIRST_ROW_OF_THE_SECOND_WINDOW = 100;
const EVERYTHING: TaskNarrowing = { channel: null, state: null, title: "" };
const ONLY_BLOCKED: TaskNarrowing = { ...EVERYTHING, state: "blocked" };

function taskRow(id: string, state = "ready"): TaskRow {
  return {
    id,
    state,
    title: `task ${id}`,
    channel: "fleet/www",
    owner: "ada",
    holder: null,
    lease: null,
    changedAt: "2026-11-21T10:27:00.000Z",
  };
}

function renderTableOf(
  rows: TaskRow[],
  readTasksThrough?: (read: ReadTasks) => ReadTasks,
) {
  const fakeCore = createFakeCore();
  let change: TaskChange | undefined;
  fakeCore.core.subscribeToTasks((latest) => {
    change = latest;
  });
  fakeCore.pushTasks(rows);
  const readTasks =
    readTasksThrough?.(fakeCore.core.readTasks) ?? fakeCore.core.readTasks;
  const onSelect = vi.fn<(taskId: string) => void>();
  const clock = createClock(ONE_SECOND);
  const table = (narrowing: TaskNarrowing) => (
    <TaskTable
      narrowing={narrowing}
      change={change}
      clock={clock}
      readTasks={readTasks}
      selectedTask={undefined}
      onSelect={onSelect}
    />
  );
  const { rerender } = render(table(EVERYTHING));
  return {
    selections: () => onSelect.mock.calls.map(([taskId]) => taskId),
    narrowTo: (narrowing: TaskNarrowing) => {
      act(() => {
        rerender(table(narrowing));
      });
    },
    pushTasks: (newRows: TaskRow[]) => {
      fakeCore.pushTasks(newRows);
      act(() => {
        rerender(table(EVERYTHING));
      });
    },
  };
}

function taskList() {
  return screen.getByRole("list", { name: dictionary.tasks.list });
}

function drawnTaskIds(): (string | undefined)[] {
  return within(taskList())
    .queryAllByRole("listitem")
    .map((row) => row.dataset.task);
}

async function rowOf(id: string): Promise<HTMLElement> {
  const title = await within(taskList()).findByText(`task ${id}`);
  const row = title.closest<HTMLElement>("li");
  if (row === null) {
    throw new Error(`the list draws no row for task ${id}`);
  }
  return row;
}

function pretendTheListIsOnScreen() {
  const scroller = taskList().parentElement as HTMLElement;
  Object.defineProperty(scroller, "clientHeight", {
    configurable: true,
    get: () => SCREEN,
  });
  return scroller;
}

function press(key: string) {
  act(() => {
    fireEvent.keyDown(document.body, { key });
  });
}

test("the rows are drawn in the order the core gives, open ones first", async () => {
  renderTableOf([
    taskRow("1", "blocked"),
    taskRow("2"),
    taskRow("3", "completed"),
  ]);

  await rowOf("3");

  expect(drawnTaskIds()).toEqual(["1", "2", "3"]);
});

test("the list has the concept's columns", () => {
  renderTableOf([]);

  const { columns } = dictionary.tasks;
  for (const name of [
    columns.state,
    columns.task,
    columns.channel,
    columns.owner,
    columns.heldBy,
    columns.updated,
  ]) {
    expect(screen.getByText(name)).toBeDefined();
  }
});

test("a board with no tasks says so", async () => {
  renderTableOf([]);

  expect(await screen.findByText(dictionary.tasks.noTasks)).toBeDefined();
});

test("a narrowing that matches no task says so", async () => {
  const table = renderTableOf([taskRow("1")]);
  await rowOf("1");

  table.narrowTo(ONLY_BLOCKED);

  expect(await screen.findByText(dictionary.tasks.noneMatch)).toBeDefined();
});

test("the rows are read in windows as the list scrolls, at any depth", async () => {
  renderTableOf(
    Array.from({ length: A_THOUSAND_TASKS }, (_, index) =>
      taskRow(String(index)),
    ),
  );
  await rowOf("0");
  const scroller = pretendTheListIsOnScreen();

  await act(async () => {
    scroller.scrollTop = HEADER_HEIGHT + DEEP_ROW * ROW_HEIGHT;
    scroller.dispatchEvent(new Event("scroll"));
    await new Promise((resolve) => requestAnimationFrame(resolve));
  });

  await rowOf(String(DEEP_ROW));
  expect(drawnTaskIds()).not.toContain("0");
  expect(drawnTaskIds().length).toBeLessThan(FEWER_THAN_A_TENTH);
});

test("the next window is read before the list scrolls to it", async () => {
  const starts: number[] = [];
  renderTableOf(
    Array.from({ length: A_THOUSAND_TASKS }, (_, index) =>
      taskRow(String(index)),
    ),
    (readTasks) => (narrowing, start, count) => {
      starts.push(start);
      return readTasks(narrowing, start, count);
    },
  );
  await rowOf("0");
  const scroller = pretendTheListIsOnScreen();

  await act(async () => {
    scroller.scrollTop = A_ROW_HALF_A_SCREEN_INTO_THE_FIRST_WINDOW * ROW_HEIGHT;
    scroller.dispatchEvent(new Event("scroll"));
    await new Promise((resolve) => requestAnimationFrame(resolve));
  });

  expect(drawnTaskIds()).not.toContain(String(FIRST_ROW_OF_THE_SECOND_WINDOW));
  expect(starts).toContain(FIRST_ROW_OF_THE_SECOND_WINDOW);
});

test("while a narrowed list is read, the rows shown before stay on screen", async () => {
  const heldReads: (() => void)[] = [];
  const table = renderTableOf(
    [taskRow("1", "blocked"), taskRow("2")],
    (readTasks) => (narrowing, start, count) => {
      if (narrowing.state === null) {
        return readTasks(narrowing, start, count);
      }
      return new Promise<TaskWindow>((resolve) => {
        heldReads.push(() => resolve(readTasks(narrowing, start, count)));
      });
    },
  );
  await rowOf("2");

  table.narrowTo(ONLY_BLOCKED);
  const whileItIsRead = drawnTaskIds();
  await act(async () => {
    for (const release of heldReads) {
      release();
    }
    await Promise.resolve();
  });

  expect(whileItIsRead).toEqual(["1", "2"]);
  await waitFor(() => expect(drawnTaskIds()).toEqual(["1"]));
});

test("a task that changes on the board changes in the list", async () => {
  const table = renderTableOf([taskRow("1")]);
  await rowOf("1");

  table.pushTasks([taskRow("1", "failed")]);

  expect(
    await within(taskList()).findByText(dictionary.tasks.stateNames.failed),
  ).toBeDefined();
});

test("clicking a row selects its task", async () => {
  const table = renderTableOf([taskRow("1"), taskRow("812")]);
  const row = await rowOf("812");

  act(() => {
    row.click();
  });

  expect(table.selections()).toEqual(["812"]);
});

test("the arrow keys move the selection through the list", async () => {
  const table = renderTableOf([taskRow("1"), taskRow("2"), taskRow("3")]);
  await rowOf("3");
  pretendTheListIsOnScreen();

  press("ArrowDown");
  press("ArrowDown");
  press("ArrowDown");
  press("ArrowDown");
  press("ArrowUp");

  expect(table.selections()).toEqual(["1", "2", "3", "2"]);
});

test("J and K move the selection as the arrows do, and the list scrolls to its row", async () => {
  const ids = Array.from({ length: ROWS_PAST_THE_SCREEN }, (_, at) => `${at}`);
  const table = renderTableOf(ids.map((id) => taskRow(id)));
  await rowOf("0");
  const scroller = pretendTheListIsOnScreen();

  for (const _ of ids) {
    press("j");
  }
  press("k");

  expect(table.selections().slice(-2)).toEqual(["24", "23"]);
  const bottomOfTheLastRow = HEADER_HEIGHT + ROWS_PAST_THE_SCREEN * ROW_HEIGHT;
  expect(scroller.scrollTop).toBe(bottomOfTheLastRow - SCREEN);
});
