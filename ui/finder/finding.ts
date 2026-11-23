import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { Command } from "../commands";
import type { SearchResults } from "../core/contract";
import { optionsOf } from "./options";
import { type Answer, createSearches, type Searches } from "./searches";

type Search = (query: string) => Promise<SearchResults>;

type State = {
  query: string;
  answered: Answer;
  selectedIndex: number;
};

const NOTHING_ANSWERED: Answer = {
  query: "",
  results: { messages: [], tasks: [], channels: [], agents: [] },
};

const OPENING_STATE: State = {
  query: "",
  answered: NOTHING_ANSWERED,
  selectedIndex: 0,
};

function wrapped(index: number, count: number): number {
  if (count === 0) {
    return 0;
  }
  return (index + count) % count;
}

function useSearchesOf(search: Search, onAnswer: (answer: Answer) => void) {
  const searches = useRef<Searches | null>(null);
  useEffect(() => {
    const created = createSearches(search, onAnswer);
    searches.current = created;
    return () => {
      created.stop();
    };
  }, [search, onAnswer]);
  return searches;
}

export function useFinding(search: Search, commands: Command[]) {
  const [state, setState] = useState(OPENING_STATE);
  const showTheAnswer = useCallback((answered: Answer) => {
    setState((current) => ({ ...current, answered, selectedIndex: 0 }));
  }, []);
  const searches = useSearchesOf(search, showTheAnswer);
  const options = useMemo(
    () => optionsOf(commands, state.query, state.answered.results),
    [commands, state.query, state.answered],
  );
  const typeQuery = useCallback(
    (typed: string) => {
      setState((current) => ({ ...current, query: typed, selectedIndex: 0 }));
      searches.current?.ask(typed);
    },
    [searches],
  );
  const count = options.length;
  const moveSelection = useCallback(
    (step: number) => {
      setState((current) => ({
        ...current,
        selectedIndex: wrapped(current.selectedIndex + step, count),
      }));
    },
    [count],
  );
  const { agents } = state.answered.results;
  const listShown =
    count + agents.length > 0 || state.answered.query.trim() !== "";
  return {
    query: state.query,
    options,
    agents,
    selectedIndex: state.selectedIndex,
    listShown,
    typeQuery,
    moveSelection,
  };
}
