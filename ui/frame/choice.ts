import { useCallback, useReducer } from "react";

type Change =
  | { kind: "lens"; lens: Lens }
  | { kind: "show"; lens: Lens }
  | { kind: "channel"; channel: string }
  | { kind: "narrow"; channel: string }
  | { kind: "widen" };

const OPENING_CHOICE: Choice = { lens: "flow", channel: null };

function choiceAfter(choice: Choice, change: Change): Choice {
  if (change.kind === "widen") {
    return { ...choice, channel: null };
  }
  if (change.kind === "narrow") {
    return { ...choice, channel: change.channel };
  }
  if (change.kind === "show") {
    return { ...choice, lens: change.lens };
  }
  if (change.kind === "lens") {
    if (change.lens === choice.lens) {
      return { ...choice, channel: null };
    }
    return { ...choice, lens: change.lens };
  }
  if (change.channel === choice.channel) {
    return { ...choice, channel: null };
  }
  return { ...choice, channel: change.channel };
}

export type Lens = "flow" | "tasks";

export type Choice = {
  lens: Lens;
  channel: string | null;
};

export function useChoice() {
  const [choice, change] = useReducer(choiceAfter, OPENING_CHOICE);
  const chooseLens = useCallback(
    (lens: Lens) => change({ kind: "lens", lens }),
    [],
  );
  const chooseChannel = useCallback(
    (channel: string) => change({ kind: "channel", channel }),
    [],
  );
  const showLens = useCallback(
    (lens: Lens) => change({ kind: "show", lens }),
    [],
  );
  const narrowTo = useCallback(
    (channel: string) => change({ kind: "narrow", channel }),
    [],
  );
  const widen = useCallback(() => change({ kind: "widen" }), []);
  return { choice, chooseLens, showLens, chooseChannel, narrowTo, widen };
}
