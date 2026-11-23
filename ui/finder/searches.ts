import type { SearchResults } from "../core/contract";

type Asked = {
  query: string;
  order: number;
};

const NOTHING_FOUND: SearchResults = {
  messages: [],
  tasks: [],
  channels: [],
  agents: [],
};

export type Answer = {
  query: string;
  results: SearchResults;
};

export type Searches = {
  ask: (query: string) => void;
  stop: () => void;
};

export function createSearches(
  search: (query: string) => Promise<SearchResults>,
  onAnswer: (answer: Answer) => void,
): Searches {
  let latestOrder = 0;
  let orderShown = 0;
  let isRunning = false;
  let waiting: Asked | undefined;
  let stopped = false;
  const show = (answer: Answer, order: number) => {
    if (stopped || order <= orderShown) {
      return;
    }
    orderShown = order;
    onAnswer(answer);
  };
  const askTheNextOne = () => {
    const next = waiting;
    waiting = undefined;
    isRunning = false;
    if (next !== undefined && !stopped) {
      run(next);
    }
  };
  const run = (asked: Asked) => {
    isRunning = true;
    search(asked.query).then(
      (results) => {
        show({ query: asked.query, results }, asked.order);
        askTheNextOne();
      },
      (error: unknown) => {
        console.error("blossom: the board could not be searched", error);
        askTheNextOne();
      },
    );
  };
  return {
    ask: (query) => {
      latestOrder += 1;
      const asked = { query, order: latestOrder };
      if (query.trim() === "") {
        waiting = undefined;
        show({ query, results: NOTHING_FOUND }, asked.order);
        return;
      }
      if (isRunning) {
        waiting = asked;
        return;
      }
      run(asked);
    },
    stop: () => {
      stopped = true;
      waiting = undefined;
    },
  };
}
