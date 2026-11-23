import {
  useCallback,
  useEffect,
  useId,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { arrowStepOf } from "../core/arrows";
import type { FoundAgent, SearchResults } from "../core/contract";
import { dictionary } from "../dictionary";
import { type FinderOption, optionsOf } from "./options";
import { FinderResults } from "./results";
import { createSearches, type Searches } from "./searches";

type Search = (query: string) => Promise<SearchResults>;

type Shown = {
  query: string;
  options: FinderOption[];
  agents: FoundAgent[];
  selectedIndex: number;
};

type BoxElement = React.RefObject<HTMLInputElement | null>;
type ScrimElement = React.RefObject<HTMLDivElement | null>;
type KeyInTheBox = React.KeyboardEvent<HTMLInputElement>;

type KeyActions = {
  moveSelection: (step: number) => void;
  openTheSelection: () => void;
  onClose: () => void;
};

const NOTHING_SHOWN: Shown = {
  query: "",
  options: [],
  agents: [],
  selectedIndex: 0,
};

function selectionMovedBy(shown: Shown, step: number): Shown {
  const count = shown.options.length;
  if (count === 0) {
    return shown;
  }
  const selectedIndex = (shown.selectedIndex + step + count) % count;
  return { ...shown, selectedIndex };
}

function useAnswersAsTheQueryChanges(search: Search) {
  const [query, setQuery] = useState("");
  const [shown, setShown] = useState(NOTHING_SHOWN);
  const searches = useRef<Searches | null>(null);
  useEffect(() => {
    const created = createSearches(search, (answer) => {
      setShown({
        query: answer.query,
        options: optionsOf(answer.results),
        agents: answer.results.agents,
        selectedIndex: 0,
      });
    });
    searches.current = created;
    return () => {
      created.stop();
    };
  }, [search]);
  const typeIntoTheBox = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const typed = event.target.value;
      setQuery(typed);
      if (searches.current !== null) {
        searches.current.ask(typed);
      }
    },
    [],
  );
  const moveSelection = useCallback((step: number) => {
    setShown((current) => selectionMovedBy(current, step));
  }, []);
  return { query, shown, typeIntoTheBox, moveSelection };
}

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
      if (option.kind === "channel") {
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

function selectedOptionId(listId: string, shown: Shown): string | undefined {
  if (shown.options[shown.selectedIndex] === undefined) {
    return undefined;
  }
  return `${listId}-${shown.selectedIndex}`;
}

function Foot() {
  const text = dictionary.finder;
  return (
    <div className="flex gap-3.5 border-line border-t px-3 py-[7px] font-mono text-[10.5px] text-ink-3">
      <span>{text.move}</span>
      <span>{text.open}</span>
      <span>{text.close}</span>
    </div>
  );
}

function QueryBox({
  box,
  listId,
  query,
  shown,
  onType,
  onKey,
}: {
  box: BoxElement;
  listId: string;
  query: string;
  shown: Shown;
  onType: (event: React.ChangeEvent<HTMLInputElement>) => void;
  onKey: (event: KeyInTheBox) => void;
}) {
  return (
    <input
      ref={box}
      role="combobox"
      aria-label={dictionary.finder.title}
      aria-expanded={shown.query.trim() !== ""}
      aria-controls={listId}
      aria-autocomplete="list"
      aria-activedescendant={selectedOptionId(listId, shown)}
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

export function FinderDialog({
  search,
  onOpenThread,
  onNarrowTo,
  onClose,
}: {
  search: Search;
  onOpenThread: (messageId: string) => void;
  onNarrowTo: (channel: string) => void;
  onClose: () => void;
}) {
  const listId = useId();
  const box = useRef<HTMLInputElement>(null);
  const scrim = useRef<HTMLDivElement>(null);
  const { query, shown, typeIntoTheBox, moveSelection } =
    useAnswersAsTheQueryChanges(search);
  const open = useOpeningOf(onOpenThread, onNarrowTo, onClose);
  const selected = shown.options[shown.selectedIndex];
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
          query={query}
          shown={shown}
          onType={typeIntoTheBox}
          onKey={takeTheKey}
        />
        {shown.query.trim() !== "" && (
          <FinderResults
            listId={listId}
            options={shown.options}
            agents={shown.agents}
            selectedIndex={shown.selectedIndex}
            onOpen={open}
          />
        )}
        <Foot />
      </div>
    </div>
  );
}
