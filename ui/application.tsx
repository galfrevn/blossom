import { useState } from "react";
import type { Clock } from "./core/clock";
import type {
  FlowChange,
  FlowWindow,
  OpenBoard,
  ReplicaState,
  SearchResults,
  TaskChange,
  TaskCount,
  Thread,
} from "./core/contract";
import { type Store, useStore } from "./core/store";
import { FinderBox } from "./finder/box";
import { FlowLens } from "./flow/lens";
import { type Lens, useChoice } from "./frame/choice";
import { Sidebar } from "./frame/sidebar";
import { StatusBar } from "./frame/status";
import { TitleBar } from "./frame/title";
import { StartScreen } from "./start";
import { TasksLens } from "./tasks/lens";
import type { ReadTasks } from "./tasks/rows";
import { createThreadOpenings, type ThreadOpenings } from "./thread/openings";

type ReadFlow = (
  channel: string | null,
  start: number,
  count: number,
) => Promise<FlowWindow>;

type CountFlow = (
  channel: string | null,
  createdAt: string,
  id: string,
) => Promise<number>;

type ReadThread = (messageId: string) => Promise<Thread | null>;

type OpenLink = (link: string) => Promise<void>;

type Search = (query: string) => Promise<SearchResults>;

function ChosenLens({
  lens,
  channel,
  boardUrl,
  flowChange,
  taskChange,
  clock,
  onWiden,
  readFlow,
  countFlowBefore,
  countFlowAfter,
  readThread,
  openLink,
  readTasks,
  readTaskCounts,
  threadOpenings,
}: {
  lens: Lens;
  channel: string | null;
  boardUrl: string;
  flowChange: FlowChange | undefined;
  taskChange: TaskChange | undefined;
  clock: Clock;
  onWiden: () => void;
  readFlow: ReadFlow;
  countFlowBefore: CountFlow;
  countFlowAfter: CountFlow;
  readThread: ReadThread;
  openLink: OpenLink;
  readTasks: ReadTasks;
  readTaskCounts: (channel: string | null) => Promise<TaskCount[]>;
  threadOpenings: ThreadOpenings;
}) {
  if (lens === "tasks") {
    return (
      <TasksLens
        change={taskChange}
        flowChange={flowChange}
        channel={channel}
        clock={clock}
        onWiden={onWiden}
        readTasks={readTasks}
        readTaskCounts={readTaskCounts}
        readThread={readThread}
        openLink={openLink}
        threadOpenings={threadOpenings}
      />
    );
  }
  return (
    <FlowLens
      change={flowChange}
      channel={channel}
      boardUrl={boardUrl}
      onWiden={onWiden}
      readFlow={readFlow}
      countFlowBefore={countFlowBefore}
      countFlowAfter={countFlowAfter}
      readThread={readThread}
      openLink={openLink}
      threadOpenings={threadOpenings}
    />
  );
}

export function Application({
  openBoardStore,
  replicaStore,
  channelsStore,
  flowStore,
  tasksStore,
  clock,
  readFlow,
  countFlowBefore,
  countFlowAfter,
  readThread,
  openLink,
  readTasks,
  readTaskCounts,
  search,
}: {
  openBoardStore: Store<OpenBoard>;
  replicaStore: Store<ReplicaState>;
  channelsStore: Store<string[]>;
  flowStore: Store<FlowChange>;
  tasksStore: Store<TaskChange>;
  clock: Clock;
  readFlow: ReadFlow;
  countFlowBefore: CountFlow;
  countFlowAfter: CountFlow;
  readThread: ReadThread;
  openLink: OpenLink;
  readTasks: ReadTasks;
  readTaskCounts: (channel: string | null) => Promise<TaskCount[]>;
  search: Search;
}) {
  const openBoard = useStore(openBoardStore);
  const replicaState = useStore(replicaStore);
  const channels = useStore(channelsStore);
  const flowChange = useStore(flowStore);
  const taskChange = useStore(tasksStore);
  const { choice, chooseLens, chooseChannel, narrowTo, widen } = useChoice();
  const [threadOpenings] = useState(createThreadOpenings);
  return (
    <div className="flex h-screen flex-col bg-paper text-ink">
      <TitleBar>
        {openBoard?.kind === "open" && (
          <FinderBox
            search={search}
            onOpenThread={threadOpenings.open}
            onNarrowTo={narrowTo}
          />
        )}
      </TitleBar>
      {openBoard?.kind === "noneRunning" && (
        <main className="flex flex-1">
          <StartScreen />
        </main>
      )}
      {openBoard?.kind === "open" && (
        <div className="flex min-h-0 flex-1">
          <Sidebar
            channels={channels ?? []}
            chosenChannel={choice.channel}
            chosenLens={choice.lens}
            onChooseChannel={chooseChannel}
            onChooseLens={chooseLens}
            taskChange={taskChange}
            readTaskCounts={readTaskCounts}
          />
          <ChosenLens
            lens={choice.lens}
            channel={choice.channel}
            boardUrl={openBoard.url}
            flowChange={flowChange}
            taskChange={taskChange}
            clock={clock}
            onWiden={widen}
            readFlow={readFlow}
            countFlowBefore={countFlowBefore}
            countFlowAfter={countFlowAfter}
            readThread={readThread}
            openLink={openLink}
            readTasks={readTasks}
            readTaskCounts={readTaskCounts}
            threadOpenings={threadOpenings}
          />
        </div>
      )}
      {openBoard?.kind === "open" && (
        <StatusBar openBoard={openBoard} replicaState={replicaState} />
      )}
    </div>
  );
}
