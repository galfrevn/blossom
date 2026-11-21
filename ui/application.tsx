import { useCallback, useState } from "react";
import type {
  FlowChange,
  FlowWindow,
  OpenBoard,
  ReplicaState,
  Thread,
} from "./core/contract";
import { type Store, useStore } from "./core/store";
import { FlowLens } from "./flow/lens";
import { Sidebar } from "./frame/sidebar";
import { StatusBar } from "./frame/status";
import { TitleBar } from "./frame/title";
import { StartScreen } from "./start";

export function Application({
  openBoardStore,
  replicaStore,
  channelsStore,
  flowStore,
  readFlow,
  readThread,
}: {
  openBoardStore: Store<OpenBoard>;
  replicaStore: Store<ReplicaState>;
  channelsStore: Store<string[]>;
  flowStore: Store<FlowChange>;
  readFlow: (
    channel: string | null,
    start: number,
    count: number,
  ) => Promise<FlowWindow>;
  readThread: (messageId: string) => Promise<Thread | null>;
}) {
  const openBoard = useStore(openBoardStore);
  const replicaState = useStore(replicaStore);
  const channels = useStore(channelsStore);
  const flowChange = useStore(flowStore);
  const [chosenChannel, setChosenChannel] = useState<string | null>(null);
  const chooseChannel = useCallback((channel: string) => {
    setChosenChannel((current) => {
      if (current === channel) {
        return null;
      }
      return channel;
    });
  }, []);
  const showTheWholeBoard = useCallback(() => setChosenChannel(null), []);
  return (
    <div className="flex h-screen flex-col bg-paper text-ink">
      <TitleBar />
      {openBoard?.kind === "noneRunning" && (
        <main className="flex flex-1">
          <StartScreen />
        </main>
      )}
      {openBoard?.kind === "open" && (
        <div className="flex min-h-0 flex-1">
          <Sidebar
            channels={channels ?? []}
            chosenChannel={chosenChannel}
            onChooseChannel={chooseChannel}
            onChooseFlow={showTheWholeBoard}
          />
          <FlowLens
            change={flowChange}
            channel={chosenChannel}
            boardUrl={openBoard.url}
            onWiden={showTheWholeBoard}
            readFlow={readFlow}
            readThread={readThread}
          />
        </div>
      )}
      {openBoard?.kind === "open" && (
        <StatusBar openBoard={openBoard} replicaState={replicaState} />
      )}
    </div>
  );
}
