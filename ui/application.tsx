import { useCallback, useState } from "react";
import { type Command, useCommands } from "./commands";
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
import { ShortcutSheet } from "./frame/sheet";
import { Sidebar } from "./frame/sidebar";
import { StatusBar } from "./frame/status";
import { TitleBar } from "./frame/title";
import { useShortcuts } from "./shortcuts";
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

const NO_COMMANDS: Command[] = [];

function commandsWhile(boardIsOpen: boolean, commands: Command[]): Command[] {
  if (!boardIsOpen) {
    return NO_COMMANDS;
  }
  return commands;
}

function useChoiceWithCommands(boardIsOpen: boolean) {
  const choosing = useChoice();
  const { choice, showLens, widen } = choosing;
  const [sheetIsOpen, setSheetIsOpen] = useState(false);
  const showShortcuts = useCallback(() => setSheetIsOpen(true), []);
  const closeTheSheet = useCallback(() => setSheetIsOpen(false), []);
  const commands = useCommands(choice, { showLens, widen, showShortcuts });
  useShortcuts(commandsWhile(boardIsOpen, commands));
  return {
    ...choosing,
    commands,
    sheet: { isOpen: boardIsOpen && sheetIsOpen, close: closeTheSheet },
  };
}

type Sheet = {
  isOpen: boolean;
  close: () => void;
};

function SheetWhenOpen({
  sheet,
  commands,
}: {
  sheet: Sheet;
  commands: Command[];
}) {
  if (!sheet.isOpen) {
    return null;
  }
  return <ShortcutSheet commands={commands} onClose={sheet.close} />;
}

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
  const [threadOpenings] = useState(createThreadOpenings);
  const choosing = useChoiceWithCommands(openBoard?.kind === "open");
  return (
    <div className="flex h-screen flex-col bg-paper text-ink">
      <TitleBar>
        {openBoard?.kind === "open" && (
          <FinderBox
            search={search}
            onOpenThread={threadOpenings.open}
            onNarrowTo={choosing.narrowTo}
            commands={choosing.commands}
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
            chosenChannel={choosing.choice.channel}
            chosenLens={choosing.choice.lens}
            onChooseChannel={choosing.chooseChannel}
            onChooseLens={choosing.chooseLens}
            taskChange={taskChange}
            readTaskCounts={readTaskCounts}
          />
          <ChosenLens
            lens={choosing.choice.lens}
            channel={choosing.choice.channel}
            boardUrl={openBoard.url}
            flowChange={flowChange}
            taskChange={taskChange}
            clock={clock}
            onWiden={choosing.widen}
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
      <SheetWhenOpen sheet={choosing.sheet} commands={choosing.commands} />
    </div>
  );
}
