import type { TaskChange, TaskCount } from "../core/contract";
import { dictionary } from "../dictionary";
import { openTaskCountIn, useTaskCounts } from "../tasks/counts";
import type { Lens } from "./choice";

function OpenTaskCount({
  id,
  counting,
}: {
  id: string;
  counting: TaskCounting;
}) {
  const counts = useTaskCounts(
    counting.channel,
    counting.change,
    counting.readTaskCounts,
  );
  if (counts === undefined) {
    return null;
  }
  const openTasks = openTaskCountIn(counts);
  return (
    <>
      <span
        aria-hidden={true}
        className="flex-none font-mono text-[11px] text-ink-3"
      >
        {dictionary.tasks.count(openTasks)}
      </span>
      <span id={id} className="sr-only">
        {dictionary.sidebar.openTasks(openTasks)}
      </span>
    </>
  );
}

export function CountBeside({
  lens,
  id,
  counting,
}: {
  lens: Lens;
  id: string;
  counting: TaskCounting;
}) {
  if (lens !== "tasks") {
    return null;
  }
  return <OpenTaskCount id={id} counting={counting} />;
}

export type TaskCounting = {
  channel: string | null;
  change: TaskChange | undefined;
  readTaskCounts: (channel: string | null) => Promise<TaskCount[]>;
};
