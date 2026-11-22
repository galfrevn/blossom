import { useEffect, useLayoutEffect, useRef } from "react";
import { arrowStepOf, comesFromTheList } from "../core/arrows";
import { recordTheScrollPosition } from "./scroll";
import { type ListState, rangeOnScreen } from "./state";

type ListElement = React.RefObject<HTMLDivElement | null>;

function messageClicked(event: MouseEvent): string | undefined {
  if (!(event.target instanceof Element)) {
    return undefined;
  }
  return event.target.closest<HTMLElement>("[data-message]")?.dataset.message;
}

function indexOfTheSelectedLine(
  state: ListState,
  selectedMessage: string | undefined,
): number | undefined {
  if (selectedMessage === undefined) {
    return undefined;
  }
  const { first, last } = rangeOnScreen(state);
  for (let index = first; index <= last; index += 1) {
    if (state.windows.rowAt(index)?.id === selectedMessage) {
      return index;
    }
  }
  return undefined;
}

function lineAtTheBottomOfTheScreen(state: ListState): number | undefined {
  if (state.heights.count === 0) {
    return undefined;
  }
  const lastPixelOnScreen = state.scrollTop + state.viewportHeight - 1;
  return state.heights.rowAt(Math.max(0, lastPixelOnScreen));
}

function indexToSelect(
  state: ListState,
  selectedMessage: string | undefined,
  step: number,
): number | undefined {
  const selectedIndex = indexOfTheSelectedLine(state, selectedMessage);
  if (selectedIndex === undefined) {
    return lineAtTheBottomOfTheScreen(state);
  }
  const next = selectedIndex + step;
  if (next < 0 || next >= state.heights.count) {
    return undefined;
  }
  return next;
}

function lineToSelect(
  state: ListState,
  selectedMessage: string | undefined,
  step: number,
) {
  const index = indexToSelect(state, selectedMessage, step);
  if (index === undefined) {
    return;
  }
  const row = state.windows.rowAt(index);
  if (row === undefined) {
    return;
  }
  return { index, messageId: row.id };
}

function bringIntoView(scroller: HTMLElement, state: ListState, index: number) {
  const top = state.heights.offsetOf(index);
  const bottom = top + state.heights.heightOf(index);
  if (top < scroller.scrollTop) {
    scroller.scrollTop = top;
  }
  if (bottom > scroller.scrollTop + scroller.clientHeight) {
    scroller.scrollTop = bottom - scroller.clientHeight;
  }
  recordTheScrollPosition(scroller, state);
}

export function useSelectionByClicks(
  content: ListElement,
  onSelect: (messageId: string) => void,
) {
  useEffect(() => {
    const element = content.current;
    if (element === null) {
      return;
    }
    const selectTheClickedLine = (event: MouseEvent) => {
      const messageId = messageClicked(event);
      if (messageId !== undefined) {
        onSelect(messageId);
      }
    };
    element.addEventListener("click", selectTheClickedLine);
    return () => {
      element.removeEventListener("click", selectTheClickedLine);
    };
  }, [content, onSelect]);
}

export function useSelectionByArrows(
  scroller: ListElement,
  state: ListState,
  selectedMessage: string | undefined,
  onSelect: (messageId: string) => void,
) {
  const latestSelection = useRef(selectedMessage);
  useLayoutEffect(() => {
    latestSelection.current = selectedMessage;
  }, [selectedMessage]);
  useEffect(() => {
    const moveTheSelection = (event: KeyboardEvent) => {
      const step = arrowStepOf(event);
      const element = scroller.current;
      if (step === undefined || element === null) {
        return;
      }
      if (!comesFromTheList(event, element)) {
        return;
      }
      event.preventDefault();
      const line = lineToSelect(state, latestSelection.current, step);
      if (line === undefined) {
        return;
      }
      latestSelection.current = line.messageId;
      bringIntoView(element, state, line.index);
      onSelect(line.messageId);
    };
    document.addEventListener("keydown", moveTheSelection);
    return () => {
      document.removeEventListener("keydown", moveTheSelection);
    };
  }, [scroller, state, onSelect]);
}
