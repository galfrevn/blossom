import { useSyncExternalStore } from "react";
import type { Core } from "./client";
import type { OpenBoard } from "./contract";

export type OpenBoardStore = {
  subscribe(onChange: () => void): () => void;
  getSnapshot(): OpenBoard | undefined;
};

export function createOpenBoardStore(core: Core): OpenBoardStore {
  let currentOpenBoard: OpenBoard | undefined;
  const changeListeners = new Set<() => void>();
  core.subscribeToOpenBoard((openBoard) => {
    currentOpenBoard = openBoard;
    for (const onChange of changeListeners) {
      onChange();
    }
  });
  return {
    subscribe(onChange) {
      changeListeners.add(onChange);
      return () => {
        changeListeners.delete(onChange);
      };
    },
    getSnapshot() {
      return currentOpenBoard;
    },
  };
}

export function useOpenBoard(store: OpenBoardStore): OpenBoard | undefined {
  return useSyncExternalStore(store.subscribe, store.getSnapshot);
}
