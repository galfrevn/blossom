import { act, cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import type { Command } from "../../commands";
import type {
  FoundAgent,
  FoundChannel,
  FoundMessage,
  FoundTask,
  SearchResults,
} from "../../core/contract";
import { dictionary } from "../../dictionary";
import { type FinderOption, optionsOf } from "../options";
import { FinderResults } from "../results";

afterEach(cleanup);

const LIST_ID = "finder-results";
const MIDDAY = "2026-11-03T12:00:00.000Z";

const foundChannel: FoundChannel = {
  path: "fleet/www",
  markedPath: [
    { text: "fleet/", marked: false },
    { text: "www", marked: true },
  ],
};

const foundTask: FoundTask = {
  id: "812",
  channel: "fleet/api",
  state: "blocked",
  snippet: [
    { text: "Ship the ", marked: false },
    { text: "www", marked: true },
    { text: " preview", marked: false },
  ],
};

const foundMessage: FoundMessage = {
  id: "47249",
  channel: "fleet/www",
  author: "claude",
  createdAt: MIDDAY,
  snippet: [
    { text: "…the ", marked: false },
    { text: "www", marked: true },
    { text: " build failed again", marked: false },
  ],
};

const foundAgent: FoundAgent = {
  id: "local:user:galfrevn+agent-1",
  markedName: [{ text: "agent-1", marked: true }],
  markedHandle: [
    { text: "galfrevn/", marked: false },
    { text: "agent-1", marked: true },
  ],
};

function resultsOf(found: Partial<SearchResults>): SearchResults {
  return { messages: [], tasks: [], channels: [], agents: [], ...found };
}

function renderResults(
  options: FinderOption[],
  selectedIndex = 0,
  onOpen: (option: FinderOption) => void = vi.fn(),
  agents: FoundAgent[] = [],
) {
  const results = (index: number) => (
    <FinderResults
      listId={LIST_ID}
      options={options}
      agents={agents}
      selectedIndex={index}
      onOpen={onOpen}
    />
  );
  const { rerender } = render(results(selectedIndex));
  return {
    select: (index: number) => {
      act(() => {
        rerender(results(index));
      });
    },
  };
}

function everyOption() {
  return screen.getAllByRole("option");
}

function markedWordsOf(option: HTMLElement) {
  return [...option.querySelectorAll("mark")].map((mark) => mark.textContent);
}

test("results are grouped: channels, tasks, messages, then agents", () => {
  renderResults(
    optionsOf(
      [],
      "",
      resultsOf({
        messages: [foundMessage],
        tasks: [foundTask],
        channels: [foundChannel],
      }),
    ),
    0,
    vi.fn(),
    [foundAgent],
  );

  const groups = screen
    .getAllByRole("group")
    .map((group) => group.getAttribute("aria-label"));

  expect(groups).toEqual([
    dictionary.finder.groups.channels,
    dictionary.finder.groups.tasks,
    dictionary.finder.groups.messages,
    dictionary.finder.groups.agents,
  ]);
});

test("a group with nothing found is left out", () => {
  renderResults(optionsOf([], "", resultsOf({ messages: [foundMessage] })));

  const groups = screen
    .getAllByRole("group")
    .map((group) => group.getAttribute("aria-label"));

  expect(groups).toEqual([dictionary.finder.groups.messages]);
});

test("in every result, the words that matched are marked", () => {
  renderResults(
    optionsOf(
      [],
      "",
      resultsOf({
        messages: [foundMessage],
        tasks: [foundTask],
        channels: [foundChannel],
      }),
    ),
  );

  expect(everyOption().map(markedWordsOf)).toEqual([["www"], ["www"], ["www"]]);
});

test("a channel reads as its path", () => {
  renderResults(optionsOf([], "", resultsOf({ channels: [foundChannel] })));

  const [option] = everyOption();

  expect(option?.textContent).toBe(dictionary.finder.channel("fleet/www"));
});

test("a task says its state and its channel", () => {
  renderResults(optionsOf([], "", resultsOf({ tasks: [foundTask] })));

  const [option] = everyOption();

  expect(option?.textContent).toContain("Ship the www preview");
  expect(option?.textContent).toContain(dictionary.tasks.stateNames.blocked);
  expect(option?.textContent).toContain(dictionary.finder.channel("fleet/api"));
});

test("a message says who wrote it, where and when", () => {
  renderResults(optionsOf([], "", resultsOf({ messages: [foundMessage] })));

  const [option] = everyOption();

  expect(option?.textContent).toContain("…the www build failed again");
  expect(option?.textContent).toContain(
    dictionary.finder.writtenBy("claude", "fleet/www", MIDDAY),
  );
});

test("the selected result is the one marked as selected", () => {
  renderResults(
    optionsOf(
      [],
      "",
      resultsOf({ tasks: [foundTask], channels: [foundChannel] }),
    ),
    1,
  );

  const selected = everyOption().map((option) =>
    option.getAttribute("aria-selected"),
  );

  expect(selected).toEqual(["false", "true"]);
  expect(everyOption()[1]?.id).toBe(`${LIST_ID}-1`);
});

test("a click on a result opens it", () => {
  const options = optionsOf(
    [],
    "",
    resultsOf({ tasks: [foundTask], channels: [foundChannel] }),
  );
  const onOpen = vi.fn();
  renderResults(options, 0, onOpen);

  act(() => {
    within(screen.getByRole("listbox"))
      .getByText(dictionary.tasks.stateNames.blocked)
      .click();
  });

  expect(onOpen).toHaveBeenCalledWith(options[1]);
});

test("the selected result is brought into view as the selection moves", () => {
  const scrolledIntoView: string[] = [];
  const scrollIntoView = vi
    .spyOn(Element.prototype, "scrollIntoView")
    .mockImplementation(function recordTheElement(this: Element) {
      scrolledIntoView.push(this.id);
    });
  const results = renderResults(
    optionsOf(
      [],
      "",
      resultsOf({ tasks: [foundTask], channels: [foundChannel] }),
    ),
  );

  results.select(1);

  expect(scrolledIntoView.at(-1)).toBe(`${LIST_ID}-1`);
  expect(scrollIntoView).toHaveBeenLastCalledWith({ block: "nearest" });
  scrollIntoView.mockRestore();
});

test("an agent reads as its name and its handle, the words that matched marked", () => {
  renderResults([], 0, vi.fn(), [foundAgent]);

  const [agent] = everyOption();

  expect(agent?.textContent).toContain("agent-1");
  expect(agent?.textContent).toContain(
    `${dictionary.finder.handleMark}galfrevn/agent-1`,
  );
  expect(everyOption().map(markedWordsOf)).toEqual([["agent-1", "agent-1"]]);
});

test("an agent is listed but opens nowhere until it has a pane", () => {
  const onOpen = vi.fn();
  renderResults([], 0, onOpen, [foundAgent]);

  const [agent] = everyOption();
  act(() => {
    agent?.click();
  });

  expect(agent?.getAttribute("aria-disabled")).toBe("true");
  expect(agent?.getAttribute("aria-selected")).toBe("false");
  expect(onOpen).not.toHaveBeenCalled();
});

test("only a query that finds nothing at all says so", () => {
  renderResults([], 0, vi.fn(), [foundAgent]);

  expect(screen.queryByText(dictionary.finder.noMatch)).toBeNull();
});

test("a command reads as its title, the words typed marked, with its shortcut beside it", () => {
  const goToTasks: Command = {
    id: "lens:tasks",
    group: "lenses",
    title: "Go to Tasks",
    shortcut: "3",
    run: vi.fn(),
  };
  renderResults(optionsOf([goToTasks], "tas", resultsOf({})));

  const [option] = everyOption();

  expect(option?.textContent).toBe("Go to Tasks3");
  expect(everyOption().map(markedWordsOf)).toEqual([["Tas"]]);
  expect(option?.querySelector("kbd")?.textContent).toBe("3");
  expect(screen.getByRole("group").getAttribute("aria-label")).toBe(
    dictionary.commands.groups.lenses,
  );
});
