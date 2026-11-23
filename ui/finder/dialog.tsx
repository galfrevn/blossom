import { useCallback, useEffect, useId, useLayoutEffect, useRef } from "react";
import type { Command } from "../commands";
import { arrowStepOf } from "../core/arrows";
import type { SearchResults } from "../core/contract";
import { dictionary } from "../dictionary";
import { useFinding } from "./finding";
import type { FinderOption } from "./options";
import { FinderResults } from "./results";

type Search = (query: string) => Promise<SearchResults>;

type BoxElement = React.RefObject<HTMLInputElement | null>;
type ScrimElement = React.RefObject<HTMLDivElement | null>;
type KeyInTheBox = React.KeyboardEvent<HTMLInputElement>;

type KeyActions = {
  moveSelection: (step: number) => void;
  openTheSelection: () => void;
  onClose: () => void;
};

type Selection = {
  listShown: boolean;
  options: FinderOption[];
  selectedIndex: number;
};

function useKeyboardInTheBox(box: BoxElement) {
  useLayoutEffect(() => {
    const focusedBefore = document.activeElement;
    box.current?.focus();
    return () => {
      if (focusedBefore instanceof HTMLElement && focusedBefore.isConnected) {
        focusedBefore.focus();
      }
    };
  }, [box]);
}

function useClosingOnClicksBeside(scrim: ScrimElement, onClose: () => void) {
  useEffect(() => {
    const element = scrim.current;
    if (element === null) {
      return;
    }
    const closeOnClicksBeside = (event: MouseEvent) => {
      if (event.target === element) {
        onClose();
      }
    };
    element.addEventListener("click", closeOnClicksBeside);
    return () => {
      element.removeEventListener("click", closeOnClicksBeside);
    };
  }, [scrim, onClose]);
}

function useOpeningOf(
  onOpenThread: (messageId: string) => void,
  onNarrowTo: (channel: string) => void,
  onClose: () => void,
) {
  return useCallback(
    (option: FinderOption) => {
      if (option.kind === "command") {
        option.command.run();
      } else if (option.kind === "channel") {
        onNarrowTo(option.found.path);
      } else {
        onOpenThread(option.found.id);
      }
      onClose();
    },
    [onOpenThread, onNarrowTo, onClose],
  );
}

function keyTakenBy(event: KeyInTheBox, actions: KeyActions): boolean {
  const step = arrowStepOf(event.nativeEvent);
  if (step !== undefined) {
    actions.moveSelection(step);
    return true;
  }
  if (event.key === "Enter") {
    actions.openTheSelection();
    return true;
  }
  if (event.key === "Escape") {
    actions.onClose();
    return true;
  }
  return event.key === "Tab";
}

function useKeysOfTheBox(actions: KeyActions) {
  const { moveSelection, openTheSelection, onClose } = actions;
  return useCallback(
    (event: KeyInTheBox) => {
      if (keyTakenBy(event, { moveSelection, openTheSelection, onClose })) {
        event.preventDefault();
        event.stopPropagation();
      }
    },
    [moveSelection, openTheSelection, onClose],
  );
}

function selectedOptionId(
  listId: string,
  selection: Selection,
): string | undefined {
  if (selection.options[selection.selectedIndex] === undefined) {
    return undefined;
  }
  return `${listId}-${selection.selectedIndex}`;
}

function Foot() {
  const text = dictionary.finder;
  return (
    <div className="flex gap-3.5 border-line border-t px-3 py-[7px] font-mono text-[10.5px] text-ink-3">
      <span>{text.move}</span>
      <span>{text.run}</span>
      <span>{text.close}</span>
    </div>
  );
}

function QueryBox({
  box,
  listId,
  query,
  selection,
  onType,
  onKey,
}: {
  box: BoxElement;
  listId: string;
  query: string;
  selection: Selection;
  onType: (event: React.ChangeEvent<HTMLInputElement>) => void;
  onKey: (event: KeyInTheBox) => void;
}) {
  return (
    <input
      ref={box}
      role="combobox"
      aria-label={dictionary.finder.title}
      aria-expanded={selection.listShown}
      aria-controls={listId}
      aria-autocomplete="list"
      aria-activedescendant={selectedOptionId(listId, selection)}
      value={query}
      onChange={onType}
      onKeyDown={onKey}
      placeholder={dictionary.finder.placeholder}
      autoComplete="off"
      spellCheck={false}
      className="block w-full border-line border-b bg-transparent px-3.5 py-3 text-[14px] text-ink outline-none placeholder:text-ink-3"
    />
  );
}

function ListWhenShown({
  listId,
  finding,
  onOpen,
}: {
  listId: string;
  finding: ReturnType<typeof useFinding>;
  onOpen: (option: FinderOption) => void;
}) {
  if (!finding.listShown) {
    return null;
  }
  return (
    <FinderResults
      listId={listId}
      options={finding.options}
      agents={finding.agents}
      selectedIndex={finding.selectedIndex}
      onOpen={onOpen}
    />
  );
}

export function FinderDialog({
  search,
  onOpenThread,
  onNarrowTo,
  onClose,
  commands,
}: {
  search: Search;
  onOpenThread: (messageId: string) => void;
  onNarrowTo: (channel: string) => void;
  onClose: () => void;
  commands: Command[];
}) {
  const listId = useId();
  const box = useRef<HTMLInputElement>(null);
  const scrim = useRef<HTMLDivElement>(null);
  const finding = useFinding(search, commands);
  const { typeQuery, moveSelection } = finding;
  const typeIntoTheBox = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      typeQuery(event.target.value);
    },
    [typeQuery],
  );
  const open = useOpeningOf(onOpenThread, onNarrowTo, onClose);
  const selected = finding.options[finding.selectedIndex];
  const openTheSelection = useCallback(() => {
    if (selected !== undefined) {
      open(selected);
    }
  }, [open, selected]);
  const takeTheKey = useKeysOfTheBox({
    moveSelection,
    openTheSelection,
    onClose,
  });
  useKeyboardInTheBox(box);
  useClosingOnClicksBeside(scrim, onClose);
  return (
    <div
      ref={scrim}
      className="fixed inset-0 z-20 flex items-start justify-center bg-scrim px-4 pt-[70px]"
    >
      <div
        role="dialog"
        aria-modal={true}
        aria-label={dictionary.finder.title}
        className="w-[min(560px,100%)] overflow-hidden rounded-[10px] border border-line-strong bg-raised shadow-[0_24px_60px_var(--shadow)]"
      >
        <QueryBox
          box={box}
          listId={listId}
          query={finding.query}
          selection={finding}
          onType={typeIntoTheBox}
          onKey={takeTheKey}
        />
        <ListWhenShown listId={listId} finding={finding} onOpen={open} />
        <Foot />
      </div>
    </div>
  );
}
