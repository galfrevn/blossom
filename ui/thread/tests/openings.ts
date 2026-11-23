import { renderHook } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { createThreadOpenings, useThreadOpenings } from "../openings";

test("each thread opened reaches whoever listens", () => {
  const openings = createThreadOpenings();
  const onOpen = vi.fn();
  renderHook(() => useThreadOpenings(openings, onOpen));

  openings.open("reply");
  openings.open("root");

  expect(onOpen.mock.calls).toEqual([["reply"], ["root"]]);
});

test("a listener that is gone hears nothing more", () => {
  const openings = createThreadOpenings();
  const onOpen = vi.fn();
  const { unmount } = renderHook(() => useThreadOpenings(openings, onOpen));

  unmount();
  openings.open("reply");

  expect(onOpen).not.toHaveBeenCalled();
});

test("a listener that comes later hears nothing opened before it", () => {
  const openings = createThreadOpenings();
  const onOpen = vi.fn();

  openings.open("reply");
  renderHook(() => useThreadOpenings(openings, onOpen));

  expect(onOpen).not.toHaveBeenCalled();
});
