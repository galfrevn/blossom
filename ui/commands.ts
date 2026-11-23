import { useMemo } from "react";
import { dictionary } from "./dictionary";
import type { Choice, Lens } from "./frame/choice";
import { LENSES } from "./frame/lenses";

type Actions = {
  showLens: (lens: Lens) => void;
  widen: () => void;
};

function lensCommands(actions: Actions): Command[] {
  return LENSES.map((entry) => ({
    id: `lens:${entry.lens}`,
    group: "lenses",
    title: dictionary.commands.goTo(entry.name),
    shortcut: entry.key,
    run: () => actions.showLens(entry.lens),
  }));
}

function wideningCommands(choice: Choice, actions: Actions): Command[] {
  const lensOnScreen = LENSES.find((entry) => entry.lens === choice.lens);
  if (choice.channel === null || lensOnScreen === undefined) {
    return [];
  }
  return [
    {
      id: "widen",
      group: "actions",
      title: dictionary.lenses.widen,
      shortcut: lensOnScreen.key,
      run: actions.widen,
    },
  ];
}

export type Command = {
  id: string;
  group: "lenses" | "actions";
  title: string;
  shortcut: string;
  run: () => void;
};

export function commandsOf(choice: Choice, actions: Actions): Command[] {
  return [...lensCommands(actions), ...wideningCommands(choice, actions)];
}

export function useCommands(
  choice: Choice,
  showLens: (lens: Lens) => void,
  widen: () => void,
): Command[] {
  return useMemo(
    () => commandsOf(choice, { showLens, widen }),
    [choice, showLens, widen],
  );
}
