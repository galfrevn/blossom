import type { Core } from "./client";
import type { OpenBoard } from "./contract";

export type FakeCore = {
  core: Core;
  push: (openBoard: OpenBoard) => void;
};

export function createFakeCore(): FakeCore {
  const listeners = new Set<(openBoard: OpenBoard) => void>();
  let latestOpenBoard: OpenBoard | undefined;
  return {
    core: {
      subscribeToOpenBoard(listener) {
        listeners.add(listener);
        if (latestOpenBoard !== undefined) {
          listener(latestOpenBoard);
        }
        return () => {
          listeners.delete(listener);
        };
      },
    },
    push(openBoard) {
      latestOpenBoard = openBoard;
      for (const listener of listeners) {
        listener(openBoard);
      }
    },
  };
}
