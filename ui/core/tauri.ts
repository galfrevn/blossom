import { Channel, invoke } from "@tauri-apps/api/core";
import type { Core } from "./client";
import type {
  Agent,
  FlowChange,
  FlowWindow,
  OpenBoard,
  ReplicaState,
  SearchResults,
  Thread,
} from "./contract";

function subscribeOverChannel<Value>(
  command: string,
  listener: (value: Value) => void,
): () => void {
  const channel = new Channel<Value>();
  let subscribed = true;
  channel.onmessage = (value) => {
    if (subscribed) {
      listener(value);
    }
  };
  invoke(command, { channel }).catch((error: unknown) => {
    console.error(`blossom: the core refused ${command}`, error);
  });
  return () => {
    subscribed = false;
  };
}

export function createTauriCore(): Core {
  return {
    subscribeToOpenBoard: (listener) =>
      subscribeOverChannel<OpenBoard>("subscribe_to_open_board", listener),
    subscribeToReplica: (listener) =>
      subscribeOverChannel<ReplicaState>("subscribe_to_replica", listener),
    subscribeToChannels: (listener) =>
      subscribeOverChannel<string[]>("subscribe_to_channels", listener),
    subscribeToFlow: (listener) =>
      subscribeOverChannel<FlowChange>("subscribe_to_flow", listener),
    subscribeToFleet: (listener) =>
      subscribeOverChannel<Agent[]>("subscribe_to_fleet", listener),
    readFlow: (channel, start, count) =>
      invoke<FlowWindow>("read_flow", { channel, start, count }),
    readThread: (messageId) =>
      invoke<Thread | null>("read_thread", { messageId }),
    search: (query) => invoke<SearchResults>("search", { query }),
    openLink: (link) => invoke<void>("open_link", { link }),
  };
}
