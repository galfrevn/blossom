import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { createClock } from "../clock";

const ONE_SECOND = 1000;
const HALF_A_SECOND = 500;
const SECONDS_WHILE_NOBODY_LISTENS = 5;
const STARTED_AT = Date.parse("2026-11-21T10:00:00.000Z");

beforeEach(() => {
  vi.useFakeTimers({ toFake: ["setInterval", "clearInterval", "Date"] });
  vi.setSystemTime(STARTED_AT);
});

afterEach(() => {
  vi.useRealTimers();
});

test("the clock tells the time it last ticked, every second while anyone listens", () => {
  const clock = createClock(ONE_SECOND);
  const ticks: number[] = [];
  clock.subscribe(() => ticks.push(clock.now()));

  vi.advanceTimersByTime(HALF_A_SECOND);
  const betweenTicks = clock.now();
  vi.advanceTimersByTime(HALF_A_SECOND + ONE_SECOND);

  expect(betweenTicks).toBe(STARTED_AT);
  expect(ticks).toEqual([STARTED_AT + ONE_SECOND, STARTED_AT + 2 * ONE_SECOND]);
});

test("the clock stops ticking once nobody listens, and tells the time again when someone does", () => {
  const clock = createClock(ONE_SECOND);
  const stopListening = clock.subscribe(() => undefined);
  stopListening();

  vi.advanceTimersByTime(ONE_SECOND * SECONDS_WHILE_NOBODY_LISTENS);
  const whileNobodyListens = vi.getTimerCount();
  clock.subscribe(() => undefined);

  expect(whileNobodyListens).toBe(0);
  expect(clock.now()).toBe(
    STARTED_AT + ONE_SECOND * SECONDS_WHILE_NOBODY_LISTENS,
  );
});
