import type { OpenBoard, ReplicaState } from "./contract";

export type Core = {
  subscribeToOpenBoard: (
    listener: (openBoard: OpenBoard) => void,
  ) => () => void;
  subscribeToReplica: (
    listener: (replicaState: ReplicaState) => void,
  ) => () => void;
  subscribeToChannels: (listener: (channels: string[]) => void) => () => void;
};
