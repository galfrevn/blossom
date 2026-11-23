import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
} from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import type { SearchResults } from "../../core/contract";
import { createFakeCore } from "../../core/fake";
import { dictionary } from "../../dictionary";
import { FinderBox } from "../box";

afterEach(cleanup);

const foundOneTask: SearchResults = {
  channels: [],
  tasks: [
    {
      id: "812",
      channel: "fleet/api",
      state: "blocked",
      snippet: [{ text: "Ship www", marked: true }],
    },
  ],
  messages: [],
  agents: [],
};

function renderTheBox() {
  const fakeCore = createFakeCore();
  fakeCore.answerSearch("www", foundOneTask);
  const opened = { threads: vi.fn(), channels: vi.fn() };
  render(
    <>
      <input aria-label="Filter by title" />
      <FinderBox
        search={fakeCore.core.search}
        onOpenThread={opened.threads}
        onNarrowTo={opened.channels}
      />
    </>,
  );
  return opened;
}

function theBox() {
  return screen.getByRole("button", { name: dictionary.finder.box });
}

function theFinder() {
  return screen.queryByRole("dialog", { name: dictionary.finder.title });
}

function pressCommandK(target: Element = document.body) {
  act(() => {
    fireEvent.keyDown(target, { key: "k", metaKey: true });
  });
}

test("the box names the finder and its shortcut", () => {
  renderTheBox();

  expect(theBox().getAttribute("aria-keyshortcuts")).toBe("Meta+K");
  expect(theBox().textContent).toContain(dictionary.finder.boxKey);
  expect(theFinder()).toBeNull();
});

test("a click on the box opens the finder", () => {
  renderTheBox();

  act(() => {
    theBox().click();
  });

  expect(theFinder()).not.toBeNull();
});

test("⌘K opens the finder, and again closes it", () => {
  renderTheBox();

  pressCommandK();
  const openedByTheKey = theFinder();
  pressCommandK(screen.getByRole("combobox"));

  expect(openedByTheKey).not.toBeNull();
  expect(theFinder()).toBeNull();
});

test("⌘K opens the finder even from a box being typed in", () => {
  renderTheBox();
  const filter = screen.getByRole("textbox", { name: "Filter by title" });
  filter.focus();

  pressCommandK(filter);

  expect(theFinder()).not.toBeNull();
});

test("K alone opens nothing", () => {
  renderTheBox();

  act(() => {
    fireEvent.keyDown(document.body, { key: "k" });
  });

  expect(theFinder()).toBeNull();
});

test("opening a result hands it over and closes the finder", async () => {
  const opened = renderTheBox();
  pressCommandK();

  fireEvent.change(screen.getByRole("combobox"), { target: { value: "www" } });
  await screen.findByRole("option");
  act(() => {
    fireEvent.keyDown(screen.getByRole("combobox"), { key: "Enter" });
  });

  expect(opened.threads).toHaveBeenCalledWith("812");
  expect(theFinder()).toBeNull();
});

test("closing the finder gives the keyboard back to where it was", () => {
  renderTheBox();
  const filter = screen.getByRole("textbox", { name: "Filter by title" });
  filter.focus();

  pressCommandK(filter);
  act(() => {
    fireEvent.keyDown(screen.getByRole("combobox"), { key: "Escape" });
  });

  expect(theFinder()).toBeNull();
  expect(document.activeElement).toBe(filter);
});
