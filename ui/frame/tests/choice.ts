import { act, renderHook } from "@testing-library/react";
import { expect, test } from "vitest";
import { useChoice } from "../choice";

function renderChoice() {
  return renderHook(() => useChoice());
}

test("the board opens on Flow, showing the whole board", () => {
  const { result } = renderChoice();

  expect(result.current.choice).toEqual({ lens: "flow", channel: null });
});

test("choosing a channel narrows to it, and choosing it again widens", () => {
  const { result } = renderChoice();

  act(() => {
    result.current.chooseChannel("fleet/www");
  });
  const narrowed = result.current.choice.channel;
  act(() => {
    result.current.chooseChannel("fleet/www");
  });

  expect(narrowed).toBe("fleet/www");
  expect(result.current.choice.channel).toBeNull();
});

test("choosing another lens keeps the channel", () => {
  const { result } = renderChoice();

  act(() => {
    result.current.chooseChannel("fleet/www");
  });
  act(() => {
    result.current.chooseLens("tasks");
  });

  expect(result.current.choice).toEqual({
    lens: "tasks",
    channel: "fleet/www",
  });
});

test("choosing the lens on screen again widens it to the whole board", () => {
  const { result } = renderChoice();

  act(() => {
    result.current.chooseChannel("fleet/www");
  });
  act(() => {
    result.current.chooseLens("flow");
  });

  expect(result.current.choice).toEqual({ lens: "flow", channel: null });
});

test("widening shows the whole board in the same lens", () => {
  const { result } = renderChoice();

  act(() => {
    result.current.chooseLens("tasks");
  });
  act(() => {
    result.current.chooseChannel("general");
  });
  act(() => {
    result.current.widen();
  });

  expect(result.current.choice).toEqual({ lens: "tasks", channel: null });
});

test("narrowing to a channel keeps it narrowed when it is already the one chosen", () => {
  const { result } = renderChoice();

  act(() => {
    result.current.narrowTo("fleet/www");
  });
  act(() => {
    result.current.narrowTo("fleet/www");
  });

  expect(result.current.choice).toEqual({
    lens: "flow",
    channel: "fleet/www",
  });
});

test("showing a lens never widens the one on screen", () => {
  const { result } = renderChoice();

  act(() => {
    result.current.narrowTo("fleet/www");
  });
  act(() => {
    result.current.showLens("flow");
  });
  const stillNarrowed = result.current.choice;
  act(() => {
    result.current.showLens("tasks");
  });

  expect(stillNarrowed).toEqual({ lens: "flow", channel: "fleet/www" });
  expect(result.current.choice).toEqual({
    lens: "tasks",
    channel: "fleet/www",
  });
});
