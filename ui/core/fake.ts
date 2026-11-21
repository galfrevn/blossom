import type { Core } from "./client";
import type { FlowChange, FlowRow, OpenBoard, ReplicaState } from "./contract";

function createFakeSubscription<Value>() {
  const listeners = new Set<(value: Value) => void>();
  let latestValue: Value | undefined;
  return {
    subscribe: (listener: (value: Value) => void) => {
      listeners.add(listener);
      if (latestValue !== undefined) {
        listener(latestValue);
      }
      return () => {
        listeners.delete(listener);
      };
    },
    push: (value: Value) => {
      latestValue = value;
      for (const listener of listeners) {
        listener(value);
      }
    },
  };
}

export type FakeCore = {
  core: Core;
  pushOpenBoard: (openBoard: OpenBoard) => void;
  pushReplicaState: (replicaState: ReplicaState) => void;
  pushChannels: (channels: string[]) => void;
  pushFlow: (rows: FlowRow[]) => void;
};

export function createFakeCore(): FakeCore {
  const openBoard = createFakeSubscription<OpenBoard>();
  const replica = createFakeSubscription<ReplicaState>();
  const channels = createFakeSubscription<string[]>();
  const flow = createFakeSubscription<FlowChange>();
  let flowRows: FlowRow[] = [];
  let flowRevision = 0;
  return {
    core: {
      subscribeToOpenBoard: openBoard.subscribe,
      subscribeToReplica: replica.subscribe,
      subscribeToChannels: channels.subscribe,
      subscribeToFlow: flow.subscribe,
      readFlow: (start, count) =>
        Promise.resolve({
          total: flowRows.length,
          start,
          rows: flowRows.slice(start, start + count),
        }),
    },
    pushOpenBoard: openBoard.push,
    pushReplicaState: replica.push,
    pushChannels: channels.push,
    pushFlow: (rows) => {
      flowRows = rows;
      flowRevision += 1;
      flow.push({ total: rows.length, revision: flowRevision });
    },
  };
}
