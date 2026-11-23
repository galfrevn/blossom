import { useEffect } from "react";

type OnOpen = (messageId: string) => void;

export type ThreadOpenings = {
  open: (messageId: string) => void;
  subscribe: (onOpen: OnOpen) => () => void;
};

export function createThreadOpenings(): ThreadOpenings {
  const listeners = new Set<OnOpen>();
  return {
    open: (messageId) => {
      for (const listener of listeners) {
        listener(messageId);
      }
    },
    subscribe: (onOpen) => {
      listeners.add(onOpen);
      return () => {
        listeners.delete(onOpen);
      };
    },
  };
}

export function useThreadOpenings(openings: ThreadOpenings, onOpen: OnOpen) {
  useEffect(() => openings.subscribe(onOpen), [openings, onOpen]);
}
