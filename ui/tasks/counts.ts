import { useEffect, useState } from "react";
import type { TaskChange, TaskCount } from "../core/contract";

const OPEN_STATES: ReadonlySet<string> = new Set([
  "blocked",
  "failed",
  "claimed",
  "ready",
  "waiting",
]);

export function useTaskCounts(
  channel: string | null,
  change: TaskChange | undefined,
  readTaskCounts: (channel: string | null) => Promise<TaskCount[]>,
): TaskCount[] | undefined {
  const [counts, setCounts] = useState<TaskCount[]>();
  const revision = change?.revision;
  useEffect(() => {
    if (revision === undefined) {
      return;
    }
    let isTheLatestRead = true;
    readTaskCounts(channel).then(
      (counted) => {
        if (isTheLatestRead) {
          setCounts(counted);
        }
      },
      (error: unknown) => {
        console.error("blossom: the tasks could not be counted", error);
      },
    );
    return () => {
      isTheLatestRead = false;
    };
  }, [channel, revision, readTaskCounts]);
  return counts;
}

export function openTaskCountIn(counts: TaskCount[]): number {
  let openTasks = 0;
  for (const count of counts) {
    if (OPEN_STATES.has(count.state)) {
      openTasks += count.count;
    }
  }
  return openTasks;
}
