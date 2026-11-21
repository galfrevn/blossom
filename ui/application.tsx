import { type OpenBoardStore, useOpenBoard } from "./core/store";
import { StatusBar } from "./frame/status";
import { TitleBar } from "./frame/title";
import { StartScreen } from "./start";

export function Application({
  openBoardStore,
}: {
  openBoardStore: OpenBoardStore;
}) {
  const openBoard = useOpenBoard(openBoardStore);
  return (
    <div className="flex h-screen flex-col bg-paper text-ink">
      <TitleBar />
      <main className="flex flex-1">
        {openBoard?.kind === "noneRunning" && <StartScreen />}
      </main>
      {openBoard?.kind === "open" && <StatusBar openBoard={openBoard} />}
    </div>
  );
}
