import { useCallback, useMemo, useState } from "react";
import type { Clock } from "../core/clock";
import type {
  FlowChange,
  TaskChange,
  TaskCount,
  TaskNarrowing,
  Thread,
} from "../core/contract";
import { dictionary } from "../dictionary";
import { LensHeader } from "../frame/lens";
import { OpenThread } from "../thread/open";
import { type ThreadOpenings, useThreadOpenings } from "../thread/openings";
import { StateChips } from "./chips";
import type { ReadTasks } from "./rows";
import { TaskTable } from "./table";

function TitleFilter({
  title,
  onChange,
}: {
  title: string;
  onChange: (title: string) => void;
}) {
  const change = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      onChange(event.target.value);
    },
    [onChange],
  );
  return (
    <input
      type="search"
      value={title}
      onChange={change}
      aria-label={dictionary.tasks.filter}
      placeholder={dictionary.tasks.filter}
      autoComplete="off"
      spellCheck={false}
      className="ml-auto w-[200px] rounded-[6px] border border-line bg-raised px-[9px] py-[3px] text-[12px] text-ink outline-none placeholder:text-ink-3 focus:border-accent-line"
    />
  );
}

export function TasksLens({
  change,
  flowChange,
  channel,
  clock,
  onWiden,
  readTasks,
  readTaskCounts,
  readThread,
  openLink,
  threadOpenings,
}: {
  change: TaskChange | undefined;
  flowChange: FlowChange | undefined;
  channel: string | null;
  clock: Clock;
  onWiden: () => void;
  readTasks: ReadTasks;
  readTaskCounts: (channel: string | null) => Promise<TaskCount[]>;
  readThread: (messageId: string) => Promise<Thread | null>;
  openLink: (link: string) => Promise<void>;
  threadOpenings: ThreadOpenings;
}) {
  const [chosenState, setChosenState] = useState<string | null>(null);
  const [title, setTitle] = useState("");
  const [selectedTask, setSelectedTask] = useState<string>();
  useThreadOpenings(threadOpenings, setSelectedTask);
  const closeTheThread = useCallback(() => {
    setSelectedTask(undefined);
  }, []);
  const narrowing: TaskNarrowing = useMemo(
    () => ({ channel, state: chosenState, title }),
    [channel, chosenState, title],
  );
  return (
    <>
      <main className="@container flex min-w-0 flex-1 flex-col">
        <LensHeader
          title={dictionary.lenses.tasks.title}
          description={dictionary.lenses.tasks.description}
          channel={channel}
          onWiden={onWiden}
        />
        <div className="flex flex-wrap items-center gap-1.5 border-line border-b px-5 py-2.5">
          <StateChips
            channel={channel}
            change={change}
            chosenState={chosenState}
            onChoose={setChosenState}
            readTaskCounts={readTaskCounts}
          />
          <TitleFilter title={title} onChange={setTitle} />
        </div>
        <TaskTable
          narrowing={narrowing}
          change={change}
          clock={clock}
          readTasks={readTasks}
          selectedTask={selectedTask}
          onSelect={setSelectedTask}
        />
      </main>
      <OpenThread
        messageId={selectedTask}
        change={flowChange}
        readThread={readThread}
        openLink={openLink}
        onClose={closeTheThread}
      />
    </>
  );
}
