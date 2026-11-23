import { useCallback } from "react";
import type { TaskChange, TaskCount } from "../core/contract";
import { dictionary } from "../dictionary";
import { useTaskCounts } from "./counts";
import { looksOf } from "./states";

function chipsToShow(
  counts: TaskCount[],
  chosenState: string | null,
): TaskCount[] {
  const shown = counts.filter(
    (count) => count.count > 0 || count.state === chosenState,
  );
  const chosenIsCounted = counts.some((count) => count.state === chosenState);
  if (chosenState === null || chosenIsCounted) {
    return shown;
  }
  return [...shown, { state: chosenState, count: 0 }];
}

function pressednessOf(isChosen: boolean): "true" | "false" {
  if (isChosen) {
    return "true";
  }
  return "false";
}

function Chip({
  count,
  isChosen,
  onChoose,
}: {
  count: TaskCount;
  isChosen: boolean;
  onChoose: (state: string | null) => void;
}) {
  const looks = looksOf(count.state);
  const choose = useCallback(() => {
    if (isChosen) {
      onChoose(null);
      return;
    }
    onChoose(count.state);
  }, [onChoose, isChosen, count.state]);
  return (
    <button
      type="button"
      aria-pressed={pressednessOf(isChosen)}
      onClick={choose}
      className="inline-flex items-center gap-[7px] rounded-[6px] border border-line bg-raised px-[9px] py-[3px] text-[12px] text-ink-2 hover:border-line-strong hover:text-ink aria-pressed:border-accent-line aria-pressed:bg-accent-wash aria-pressed:text-ink"
    >
      <i
        aria-hidden={true}
        className={`size-[7px] rounded-[1px] ${looks.swatch}`}
      />
      {looks.name}{" "}
      <b className="font-mono font-normal text-[11px] text-ink-3">
        {dictionary.tasks.count(count.count)}
      </b>
    </button>
  );
}

export function StateChips({
  channel,
  change,
  chosenState,
  onChoose,
  readTaskCounts,
}: {
  channel: string | null;
  change: TaskChange | undefined;
  chosenState: string | null;
  onChoose: (state: string | null) => void;
  readTaskCounts: (channel: string | null) => Promise<TaskCount[]>;
}) {
  const counts = useTaskCounts(channel, change, readTaskCounts) ?? [];
  return (
    <fieldset
      aria-label={dictionary.tasks.states}
      className="flex min-w-0 flex-wrap gap-1.5"
    >
      {chipsToShow(counts, chosenState).map((count) => (
        <Chip
          key={count.state}
          count={count}
          isChosen={count.state === chosenState}
          onChoose={onChoose}
        />
      ))}
    </fieldset>
  );
}
