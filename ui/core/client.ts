import type { OpenBoard } from "./contract";

export type Core = {
  subscribeToOpenBoard: (
    listener: (openBoard: OpenBoard) => void,
  ) => () => void;
};
