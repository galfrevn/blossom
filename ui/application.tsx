import type { OpenBoard, ReplicaState } from "./core/contract";
import { type Store, useStore } from "./core/store";
import { StatusBar } from "./frame/status";
import { TitleBar } from "./frame/title";
import { StartScreen } from "./start";

export function Application({
  openBoardStore,
  replicaStore,
}: {
  openBoardStore: Store<OpenBoard>;
  replicaStore: Store<ReplicaState>;
}) {
  const openBoard = useStore(openBoardStore);
  const replicaState = useStore(replicaStore);
  return (
    <div className="flex h-screen flex-col bg-paper text-ink">
      <TitleBar />
      <main className="flex flex-1">
        {openBoard?.kind === "noneRunning" && <StartScreen />}
      </main>
      {openBoard?.kind === "open" && (
        <StatusBar openBoard={openBoard} replicaState={replicaState} />
      )}
    </div>
  );
}
