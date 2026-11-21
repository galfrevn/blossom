import type {
  FlowChange,
  FlowWindow,
  OpenBoard,
  ReplicaState,
} from "./contract";

export type Core = {
  subscribeToOpenBoard: (
    listener: (openBoard: OpenBoard) => void,
  ) => () => void;
  subscribeToReplica: (
    listener: (replicaState: ReplicaState) => void,
  ) => () => void;
  subscribeToChannels: (listener: (channels: string[]) => void) => () => void;
  subscribeToFlow: (listener: (change: FlowChange) => void) => () => void;
  readFlow: (start: number, count: number) => Promise<FlowWindow>;
};
