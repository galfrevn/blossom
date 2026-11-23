import { expect, test, vi } from "vitest";
import type { SearchResults } from "../../core/contract";
import { type Answer, createSearches } from "../searches";

type Asked = {
  query: string;
  answer: (results: SearchResults) => Promise<void>;
  refuse: () => Promise<void>;
};

function resultsFor(query: string): SearchResults {
  return {
    messages: [
      {
        id: `message-${query}`,
        channel: "fleet/www",
        author: "claude",
        createdAt: "2026-11-21T10:00:00.000Z",
        snippet: [{ text: query, marked: true }],
      },
    ],
    tasks: [],
    channels: [],
    agents: [],
  };
}

function settle() {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

function searchesOverHeldAnswers() {
  const asked: Asked[] = [];
  const answers: Answer[] = [];
  const search = (query: string) =>
    new Promise<SearchResults>((resolve, reject) => {
      asked.push({
        query,
        answer: async (results) => {
          resolve(results);
          await settle();
        },
        refuse: async () => {
          reject(new Error("no replica is open"));
          await settle();
        },
      });
    });
  const searches = createSearches(search, (answer) => {
    answers.push(answer);
  });
  const askedAt = (index: number): Asked => {
    const found = asked[index];
    if (found === undefined) {
      throw new Error(`the core was asked only ${asked.length} times`);
    }
    return found;
  };
  return {
    searches,
    askedQueries: () => asked.map((entry) => entry.query),
    askedAt,
    answeredQueries: () => answers.map((answer) => answer.query),
    answers,
  };
}

test("a query is asked of the core at once, and its answer is shown", async () => {
  const held = searchesOverHeldAnswers();

  held.searches.ask("deploy");
  await held.askedAt(0).answer(resultsFor("deploy"));

  expect(held.askedQueries()).toEqual(["deploy"]);
  expect(held.answers).toEqual([
    { query: "deploy", results: resultsFor("deploy") },
  ]);
});

test("while a query runs, only the latest of the queries typed after it waits", async () => {
  const held = searchesOverHeldAnswers();

  held.searches.ask("d");
  held.searches.ask("de");
  held.searches.ask("dep");
  held.searches.ask("depl");

  expect(held.askedQueries()).toEqual(["d"]);

  await held.askedAt(0).answer(resultsFor("d"));

  expect(held.askedQueries()).toEqual(["d", "depl"]);
});

test("the answer to a slow query is shown while the latest one runs", async () => {
  const held = searchesOverHeldAnswers();

  held.searches.ask("d");
  held.searches.ask("deploy");
  await held.askedAt(0).answer(resultsFor("d"));

  expect(held.answeredQueries()).toEqual(["d"]);

  await held.askedAt(1).answer(resultsFor("deploy"));

  expect(held.answeredQueries()).toEqual(["d", "deploy"]);
});

test("an empty query answers nothing at once, and drops the query still running", async () => {
  const held = searchesOverHeldAnswers();

  held.searches.ask("deploy");
  held.searches.ask("deploy www");
  held.searches.ask("  ");
  await held.askedAt(0).answer(resultsFor("deploy"));

  expect(held.askedQueries()).toEqual(["deploy"]);
  expect(held.answers).toEqual([
    {
      query: "  ",
      results: { messages: [], tasks: [], channels: [], agents: [] },
    },
  ]);
});

test("a query the core refuses leaves what is shown, and the next query is asked", async () => {
  const held = searchesOverHeldAnswers();
  const reported = vi
    .spyOn(console, "error")
    .mockImplementation(() => undefined);

  held.searches.ask("deploy");
  held.searches.ask("deploy www");
  await held.askedAt(0).refuse();
  await held.askedAt(1).answer(resultsFor("deploy www"));

  expect(held.answeredQueries()).toEqual(["deploy www"]);
  expect(reported).toHaveBeenCalledOnce();
  reported.mockRestore();
});

test("once stopped, no answer is shown and nothing more is asked", async () => {
  const held = searchesOverHeldAnswers();

  held.searches.ask("deploy");
  held.searches.ask("deploy www");
  held.searches.stop();
  await held.askedAt(0).answer(resultsFor("deploy"));

  expect(held.askedQueries()).toEqual(["deploy"]);
  expect(held.answers).toEqual([]);
});
