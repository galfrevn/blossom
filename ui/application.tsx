import type { Clock } from "./core/clock";
import type {
  FlowChange,
  FlowWindow,
  OpenBoard,
  ReplicaState,
  TaskChange,
  TaskCount,
  Thread,
} from "./core/contract";
import { type Store, useStore } from "./core/store";
import { FlowLens } from "./flow/lens";
import { type Lens, useChoice } from "./frame/choice";
import { Sidebar } from "./frame/sidebar";
import { StatusBar } from "./frame/status";
import { TitleBar } from "./frame/title";
import { StartScreen } from "./start";
import { TasksLens } from "./tasks/lens";
import type { ReadTasks } from "./tasks/rows";

type ReadFlow = (
  channel: string | null,
  start: number,
  count: number,
) => Promise<FlowWindow>;

type CountFlowAfter = (
  channel: string | null,
  createdAt: string,
  id: string,
) => Promise<number>;

type ReadThread = (messageId: string) => Promise<Thread | null>;

type OpenLink = (link: string) => Promise<void>;

function ChosenLens({
  lens,
  channel,
  boardUrl,
  flowChange,
  taskChange,
  clock,
  onWiden,
  readFlow,
  countFlowAfter,
  readThread,
  openLink,
  readTasks,
  readTaskCounts,
}: {
  lens: Lens;
  channel: string | null;
  boardUrl: string;
  flowChange: FlowChange | undefined;
  taskChange: TaskChange | undefined;
  clock: Clock;
  onWiden: () => void;
  readFlow: ReadFlow;
  countFlowAfter: CountFlowAfter;
  readThread: ReadThread;
  openLink: OpenLink;
  readTasks: ReadTasks;
  readTaskCounts: (channel: string | null) => Promise<TaskCount[]>;
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
      countFlowAfter={countFlowAfter}
      readThread={readThread}
      openLink={openLink}
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
  countFlowAfter,
  readThread,
  openLink,
  readTasks,
  readTaskCounts,
}: {
  openBoardStore: Store<OpenBoard>;
  replicaStore: Store<ReplicaState>;
  channelsStore: Store<string[]>;
  flowStore: Store<FlowChange>;
  tasksStore: Store<TaskChange>;
  clock: Clock;
  readFlow: ReadFlow;
  countFlowAfter: CountFlowAfter;
  readThread: ReadThread;
  openLink: OpenLink;
  readTasks: ReadTasks;
  readTaskCounts: (channel: string | null) => Promise<TaskCount[]>;
}) {
  const openBoard = useStore(openBoardStore);
  const replicaState = useStore(replicaStore);
  const channels = useStore(channelsStore);
  const flowChange = useStore(flowStore);
  const taskChange = useStore(tasksStore);
  const { choice, chooseLens, chooseChannel, widen } = useChoice();
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
            countFlowAfter={countFlowAfter}
            readThread={readThread}
            openLink={openLink}
            readTasks={readTasks}
            readTaskCounts={readTaskCounts}
          />
        </div>
      )}
      {openBoard?.kind === "open" && (
        <StatusBar openBoard={openBoard} replicaState={replicaState} />
      )}
    </div>
  );
}
