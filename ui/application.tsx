import type { OpenBoard, ReplicaState } from "./core/contract";
import { type Store, useStore } from "./core/store";
import { dictionary } from "./dictionary";
import { LensHeader } from "./frame/lens";
import { Sidebar } from "./frame/sidebar";
import { StatusBar } from "./frame/status";
import { TitleBar } from "./frame/title";
import { StartScreen } from "./start";

export function Application({
  openBoardStore,
  replicaStore,
  channelsStore,
}: {
  openBoardStore: Store<OpenBoard>;
  replicaStore: Store<ReplicaState>;
  channelsStore: Store<string[]>;
}) {
  const openBoard = useStore(openBoardStore);
  const replicaState = useStore(replicaStore);
  const channels = useStore(channelsStore);
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
          <Sidebar channels={channels ?? []} />
          <main className="flex min-w-0 flex-1 flex-col">
            <LensHeader
              title={dictionary.lenses.flow.title}
              description={dictionary.lenses.flow.description}
            />
          </main>
        </div>
      )}
      {openBoard?.kind === "open" && (
        <StatusBar openBoard={openBoard} replicaState={replicaState} />
      )}
    </div>
  );
}
