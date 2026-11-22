import { useSyncExternalStore } from "react";

export type Clock = {
  subscribe: (onTick: () => void) => () => void;
  now: () => number;
};

export function createClock(millisecondsBetweenTicks: number): Clock {
  let lastTick = Date.now();
  let ticking: ReturnType<typeof setInterval> | undefined;
  const listeners = new Set<() => void>();
  const tick = () => {
    lastTick = Date.now();
    for (const listener of listeners) {
      listener();
    }
  };
  const stopWhenNobodyListens = () => {
    if (listeners.size > 0 || ticking === undefined) {
      return;
    }
    clearInterval(ticking);
    ticking = undefined;
  };
  return {
    subscribe: (onTick) => {
      listeners.add(onTick);
      if (ticking === undefined) {
        lastTick = Date.now();
        ticking = setInterval(tick, millisecondsBetweenTicks);
      }
      return () => {
        listeners.delete(onTick);
        stopWhenNobodyListens();
      };
    },
    now: () => lastTick,
  };
}

export function useReadingOf<Reading>(
  clock: Clock,
  read: (now: number) => Reading,
): Reading {
  return useSyncExternalStore(clock.subscribe, () => read(clock.now()));
}
