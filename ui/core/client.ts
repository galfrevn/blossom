import type {
  Agent,
  FlowChange,
  FlowWindow,
  OpenBoard,
  ReplicaState,
  Thread,
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
  subscribeToFleet: (listener: (agents: Agent[]) => void) => () => void;
  readFlow: (
    channel: string | null,
    start: number,
    count: number,
  ) => Promise<FlowWindow>;
  readThread: (messageId: string) => Promise<Thread | null>;
};
