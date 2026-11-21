import { useEffect, useLayoutEffect } from "react";
import { isAtTheBottom, type ListState } from "./state";
import { WatchedRows } from "./watch";

function heightStillToMeasure(
  element: HTMLElement,
  state: ListState,
  index: number,
): number {
  if (state.heights.isMeasured(index)) {
    return 0;
  }
  return element.offsetHeight;
}

function measure(scroller: HTMLElement, state: ListState) {
  let shiftAbove = 0;
  let changed = false;
  for (const element of scroller.querySelectorAll<HTMLElement>(
    "[data-loaded]",
  )) {
    const index = Number(element.dataset.index);
    const height = heightStillToMeasure(element, state, index);
    if (height > 0) {
      const top = state.heights.offsetOf(index);
      const change = state.heights.set(index, height);
      if (change !== 0) {
        changed = true;
        if (top < state.scrollTop) {
          shiftAbove += change;
        }
      }
    }
  }
  for (const element of scroller.querySelectorAll<HTMLElement>(
    "[data-placeholder]",
  )) {
    const index = Number(element.dataset.index);
    element.style.height = `${state.heights.heightOf(index)}px`;
  }
  return { shiftAbove, changed };
}

function place(scroller: HTMLElement, state: ListState) {
  for (const element of scroller.querySelectorAll<HTMLElement>(
    "[data-index]",
  )) {
    const top = state.heights.offsetOf(Number(element.dataset.index));
    element.style.transform = `translateY(${top}px)`;
  }
}

export function useScrollFollowing(
  scrollerRef: React.RefObject<HTMLDivElement | null>,
  state: ListState,
  redraw: () => void,
) {
  useEffect(() => {
    const scroller = scrollerRef.current;
    if (scroller === null) {
      return;
    }
    let frame = 0;
    const onScroll = () => {
      if (frame !== 0) {
        return;
      }
      frame = requestAnimationFrame(() => {
        frame = 0;
        state.scrollTop = scroller.scrollTop;
        state.viewportHeight = scroller.clientHeight;
        const atTheBottom = isAtTheBottom(scroller);
        if (state.keepsToTheBottom && !atTheBottom) {
          state.totalWhenItLeftTheBottom = state.heights.count;
        }
        state.keepsToTheBottom = atTheBottom;
        redraw();
      });
    };
    state.viewportHeight = scroller.clientHeight;
    scroller.addEventListener("scroll", onScroll, { passive: true });
    let width = scroller.clientWidth;
    const resizing = new ResizeObserver((entries) => {
      if (scroller.clientWidth !== width) {
        width = scroller.clientWidth;
        state.heights.forgetMeasurements();
        redraw();
        return;
      }
      let rowChanged = false;
      for (const entry of entries) {
        const index = Number((entry.target as HTMLElement).dataset.index);
        const height = (entry.target as HTMLElement).offsetHeight;
        const changedItsHeight =
          entry.target !== scroller &&
          height > 0 &&
          height !== state.heights.heightOf(index);
        if (changedItsHeight) {
          state.heights.forgetMeasurementOf(index);
          rowChanged = true;
        }
      }
      if (rowChanged) {
        redraw();
      }
    });
    resizing.observe(scroller);
    state.watchedRows = new WatchedRows(resizing);
    return () => {
      cancelAnimationFrame(frame);
      resizing.disconnect();
      state.watchedRows = null;
      scroller.removeEventListener("scroll", onScroll);
    };
  }, [scrollerRef, state, redraw]);
}

export type ListElements = {
  scroller: React.RefObject<HTMLDivElement | null>;
  content: React.RefObject<HTMLDivElement | null>;
};

export function useLayoutAfterDrawing(
  elements: ListElements,
  state: ListState,
  redraw: () => void,
) {
  useLayoutEffect(() => {
    const scroller = elements.scroller.current;
    const content = elements.content.current;
    if (scroller === null || content === null) {
      return;
    }
    const { shiftAbove, changed } = measure(scroller, state);
    state.watchedRows?.follow(scroller.querySelectorAll("[data-loaded]"));
    const { heights } = state;
    content.style.height = `${heights.total()}px`;
    place(scroller, state);
    if (state.keepsToTheBottom && heights.count > 0) {
      scroller.scrollTop = scroller.scrollHeight;
    } else if (shiftAbove !== 0) {
      scroller.scrollTop += shiftAbove;
    }
    const scrollMoved = scroller.scrollTop !== state.scrollTop;
    state.scrollTop = scroller.scrollTop;
    state.viewportHeight = scroller.clientHeight;
    if (changed || scrollMoved) {
      redraw();
    }
  });
}
