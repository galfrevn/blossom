import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import type { SearchResults } from "../../core/contract";
import { createFakeCore } from "../../core/fake";
import { dictionary } from "../../dictionary";
import { FinderDialog } from "../dialog";

afterEach(cleanup);

const foundEverywhere: SearchResults = {
  channels: [
    {
      path: "fleet/www",
      markedPath: [
        { text: "fleet/", marked: false },
        { text: "www", marked: true },
      ],
    },
  ],
  tasks: [
    {
      id: "812",
      channel: "fleet/api",
      state: "blocked",
      snippet: [{ text: "Ship www", marked: false }],
    },
  ],
  messages: [
    {
      id: "47249",
      channel: "fleet/www",
      author: "claude",
      createdAt: "2026-11-03T12:00:00.000Z",
      snippet: [{ text: "www build failed", marked: false }],
    },
  ],
  agents: [
    {
      id: "local:user:galfrevn+www",
      markedName: [{ text: "www", marked: true }],
      markedHandle: [{ text: "galfrevn/www", marked: false }],
    },
  ],
};

function foundOneMessage(text: string): SearchResults {
  return {
    channels: [],
    tasks: [],
    messages: [
      {
        id: `message ${text}`,
        channel: "fleet/www",
        author: "claude",
        createdAt: "2026-11-03T12:00:00.000Z",
        snippet: [{ text, marked: true }],
      },
    ],
    agents: [],
  };
}

function renderFinderOver(search: (query: string) => Promise<SearchResults>) {
  const opened = {
    threads: vi.fn(),
    channels: vi.fn(),
    closings: vi.fn(),
  };
  render(
    <FinderDialog
      search={search}
      onOpenThread={opened.threads}
      onNarrowTo={opened.channels}
      onClose={opened.closings}
    />,
  );
  return opened;
}

function renderFinderOverTheFakeCore() {
  const fakeCore = createFakeCore();
  fakeCore.answerSearch("www", foundEverywhere);
  return renderFinderOver(fakeCore.core.search);
}

function theBox() {
  return screen.getByRole("combobox", { name: dictionary.finder.title });
}

function typeIntoTheBox(query: string) {
  fireEvent.change(theBox(), { target: { value: query } });
}

function press(key: string) {
  let wentOn = true;
  act(() => {
    wentOn = fireEvent.keyDown(theBox(), { key });
  });
  return wentOn;
}

async function searchFor(query: string) {
  typeIntoTheBox(query);
  return await screen.findAllByRole("option");
}

function markedInTheSelection() {
  const selected = screen
    .getAllByRole("option")
    .find((option) => option.getAttribute("aria-selected") === "true");
  const marks = selected?.querySelectorAll("mark") ?? [];
  return [...marks].map((mark) => mark.textContent);
}

function selectedOptionText() {
  const selected = screen
    .getAllByRole("option")
    .filter((option) => option.getAttribute("aria-selected") === "true");
  return selected.map((option) => option.textContent);
}

test("the finder opens with the keyboard in its box", () => {
  renderFinderOverTheFakeCore();

  expect(
    screen.getByRole("dialog", { name: dictionary.finder.title }),
  ).toBeDefined();
  expect(document.activeElement).toBe(theBox());
});

test("typing searches the replica and lists what it found", async () => {
  renderFinderOverTheFakeCore();

  const options = await searchFor("www");

  expect(options.map((option) => option.id)).toHaveLength(
    foundEverywhere.channels.length +
      foundEverywhere.tasks.length +
      foundEverywhere.messages.length +
      foundEverywhere.agents.length,
  );
  expect(theBox().getAttribute("aria-expanded")).toBe("true");
});

test("the first result is selected, and the box says which", async () => {
  renderFinderOverTheFakeCore();

  const [first] = await searchFor("www");

  expect(first?.getAttribute("aria-selected")).toBe("true");
  expect(theBox().getAttribute("aria-activedescendant")).toBe(first?.id);
});

test("the arrows move the selection, go round at either end, and pass over the agents", async () => {
  renderFinderOverTheFakeCore();
  const options = await searchFor("www");
  const texts = options.map((option) => option.textContent);

  press("ArrowDown");
  const second = selectedOptionText();
  press("ArrowDown");
  press("ArrowDown");
  const roundToTheFirst = selectedOptionText();
  press("ArrowUp");

  expect(second).toEqual([texts[1]]);
  expect(roundToTheFirst).toEqual([texts[0]]);
  expect(selectedOptionText()).toEqual([texts[2]]);
});

test("Enter on a channel narrows the lens to it, and the finder closes", async () => {
  const opened = renderFinderOverTheFakeCore();
  await searchFor("www");

  press("Enter");

  expect(opened.channels).toHaveBeenCalledWith("fleet/www");
  expect(opened.threads).not.toHaveBeenCalled();
  expect(opened.closings).toHaveBeenCalledOnce();
});

test("Enter on a task or a message opens its thread", async () => {
  const opened = renderFinderOverTheFakeCore();
  await searchFor("www");

  press("ArrowDown");
  press("Enter");
  press("ArrowDown");
  press("Enter");

  expect(opened.threads.mock.calls).toEqual([["812"], ["47249"]]);
});

test("a click on a result opens it as Enter does", async () => {
  const opened = renderFinderOverTheFakeCore();
  await searchFor("www");

  act(() => {
    screen.getByText("www build failed").click();
  });

  expect(opened.threads).toHaveBeenCalledWith("47249");
  expect(opened.closings).toHaveBeenCalledOnce();
});

test("Esc closes the finder, and the keys it takes never reach the app behind", async () => {
  const opened = renderFinderOverTheFakeCore();
  await searchFor("www");
  const reachedTheApp = vi.fn();
  document.addEventListener("keydown", reachedTheApp);

  press("ArrowDown");
  press("Enter");
  press("Escape");

  document.removeEventListener("keydown", reachedTheApp);
  expect(reachedTheApp).not.toHaveBeenCalled();
  expect(opened.closings).toHaveBeenCalled();
});

test("Tab keeps the keyboard in the box", () => {
  renderFinderOverTheFakeCore();

  expect(press("Tab")).toBe(false);
});

test("a query that finds nothing says so", async () => {
  renderFinderOverTheFakeCore();

  typeIntoTheBox("zzz");

  expect(await screen.findByText(dictionary.finder.noMatch)).toBeDefined();
});

test("an empty box lists nothing", async () => {
  renderFinderOverTheFakeCore();
  await searchFor("www");

  typeIntoTheBox("");

  expect(await screen.findByRole("combobox")).toBeDefined();
  expect(screen.queryByRole("listbox")).toBeNull();
  expect(theBox().getAttribute("aria-expanded")).toBe("false");
});

test("the results follow the typing, without waiting on a slow query", async () => {
  const answers = new Map<string, (results: SearchResults) => void>();
  const search = (query: string) =>
    new Promise<SearchResults>((resolve) => {
      answers.set(query, resolve);
    });
  renderFinderOver(search);

  typeIntoTheBox("b");
  typeIntoTheBox("bu");
  typeIntoTheBox("build");
  await act(async () => {
    answers.get("b")?.(foundOneMessage("b"));
    await Promise.resolve();
  });
  const whileTheLatestRuns = markedInTheSelection();
  await act(async () => {
    answers.get("build")?.(foundOneMessage("build"));
    await Promise.resolve();
  });

  expect([...answers.keys()]).toEqual(["b", "build"]);
  expect(whileTheLatestRuns).toEqual(["b"]);
  expect(markedInTheSelection()).toEqual(["build"]);
});

test("a click beside the box closes the finder, and a click in it does not", () => {
  const opened = renderFinderOverTheFakeCore();
  const dialog = screen.getByRole("dialog");

  act(() => {
    within(dialog).getByRole("combobox").click();
  });
  const closedFromInside = opened.closings.mock.calls.length;
  act(() => {
    dialog.parentElement?.click();
  });

  expect(closedFromInside).toBe(0);
  expect(opened.closings).toHaveBeenCalledOnce();
});
