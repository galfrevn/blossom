import { useEffect, useLayoutEffect, useRef } from "react";
import { comesFromTheList, listStepOf } from "../core/arrows";
import {
  HEADER_HEIGHT,
  ROW_HEIGHT,
  rangeOnScreen,
  shownRows,
  type TableState,
} from "./rows";

type ListElement = React.RefObject<HTMLElement | null>;

function taskClicked(event: MouseEvent): string | undefined {
  if (!(event.target instanceof Element)) {
    return undefined;
  }
  return event.target.closest<HTMLElement>("[data-task]")?.dataset.task;
}

function indexOfTheSelectedRow(
  state: TableState,
  selectedTask: string | undefined,
): number | undefined {
  if (selectedTask === undefined) {
    return undefined;
  }
  const { windows } = shownRows(state);
  const { first, last } = rangeOnScreen(state, windows.total());
  for (let index = first; index <= last; index += 1) {
    if (windows.rowAt(index)?.id === selectedTask) {
      return index;
    }
  }
  return undefined;
}

function firstRowOnScreen(state: TableState): number {
  return Math.floor(state.scrollTop / ROW_HEIGHT);
}

function indexToSelect(
  state: TableState,
  selectedTask: string | undefined,
  step: number,
): number | undefined {
  const total = shownRows(state).windows.total() ?? 0;
  if (total === 0) {
    return undefined;
  }
  const selectedIndex = indexOfTheSelectedRow(state, selectedTask);
  if (selectedIndex === undefined) {
    return Math.min(firstRowOnScreen(state), total - 1);
  }
  const next = selectedIndex + step;
  if (next < 0 || next >= total) {
    return undefined;
  }
  return next;
}

function bringIntoView(
  scroller: HTMLElement,
  state: TableState,
  index: number,
) {
  const top = index * ROW_HEIGHT;
  const bottom = HEADER_HEIGHT + top + ROW_HEIGHT;
  if (top < scroller.scrollTop) {
    scroller.scrollTop = top;
  }
  if (bottom > scroller.scrollTop + scroller.clientHeight) {
    scroller.scrollTop = bottom - scroller.clientHeight;
  }
  state.scrollTop = scroller.scrollTop;
  state.viewportHeight = scroller.clientHeight;
}

export function useSelectionByClicks(
  rows: ListElement,
  onSelect: (taskId: string) => void,
) {
  useEffect(() => {
    const element = rows.current;
    if (element === null) {
      return;
    }
    const selectTheClickedTask = (event: MouseEvent) => {
      const taskId = taskClicked(event);
      if (taskId !== undefined) {
        onSelect(taskId);
      }
    };
    element.addEventListener("click", selectTheClickedTask);
    return () => {
      element.removeEventListener("click", selectTheClickedTask);
    };
  }, [rows, onSelect]);
}

export function useSelectionByArrows(
  scroller: ListElement,
  state: TableState,
  selectedTask: string | undefined,
  onSelect: (taskId: string) => void,
) {
  const latestSelection = useRef(selectedTask);
  useLayoutEffect(() => {
    latestSelection.current = selectedTask;
  }, [selectedTask]);
  useEffect(() => {
    const moveTheSelection = (event: KeyboardEvent) => {
      const step = listStepOf(event);
      const element = scroller.current;
      if (step === undefined || element === null) {
        return;
      }
      if (!comesFromTheList(event, element)) {
        return;
      }
      event.preventDefault();
      const index = indexToSelect(state, latestSelection.current, step);
      if (index === undefined) {
        return;
      }
      const row = shownRows(state).windows.rowAt(index);
      if (row === undefined) {
        return;
      }
      latestSelection.current = row.id;
      bringIntoView(element, state, index);
      onSelect(row.id);
    };
    document.addEventListener("keydown", moveTheSelection);
    return () => {
      document.removeEventListener("keydown", moveTheSelection);
    };
  }, [scroller, state, onSelect]);
}
