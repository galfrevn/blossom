import { dictionary } from "../dictionary";

const NAMES = dictionary.tasks.stateNames;

const LOOKS_OF_EACH_STATE: ReadonlyMap<string, StateLooks> = new Map([
  [
    "blocked",
    {
      name: NAMES.blocked,
      ink: "text-state-blocked",
      swatch: "bg-state-blocked",
    },
  ],
  [
    "failed",
    { name: NAMES.failed, ink: "text-state-failed", swatch: "bg-state-failed" },
  ],
  [
    "claimed",
    {
      name: NAMES.claimed,
      ink: "text-state-claimed",
      swatch: "bg-state-claimed",
    },
  ],
  [
    "ready",
    { name: NAMES.ready, ink: "text-state-ready", swatch: "bg-state-ready" },
  ],
  [
    "waiting",
    {
      name: NAMES.waiting,
      ink: "text-state-waiting",
      swatch: "bg-state-waiting",
    },
  ],
  [
    "completed",
    {
      name: NAMES.completed,
      ink: "text-state-completed",
      swatch: "bg-state-completed",
    },
  ],
  [
    "canceled",
    {
      name: NAMES.canceled,
      ink: "text-state-canceled line-through",
      swatch: "bg-state-canceled",
    },
  ],
]);

export type StateLooks = {
  name: string;
  ink: string;
  swatch: string;
};

export function looksOf(state: string): StateLooks {
  return (
    LOOKS_OF_EACH_STATE.get(state) ?? {
      name: state,
      ink: "text-ink-3",
      swatch: "bg-ink-3",
    }
  );
}
