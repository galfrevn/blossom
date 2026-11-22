import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import type { Lens } from "../choice";
import { Sidebar } from "../sidebar";

afterEach(cleanup);

const nothing = () => undefined;

function renderSidebar(
  channels: string[],
  chosenChannel: string | null = null,
  chosenLens: Lens = "flow",
) {
  return render(
    <Sidebar
      channels={channels}
      chosenChannel={chosenChannel}
      chosenLens={chosenLens}
      onChooseChannel={nothing}
      onChooseLens={nothing}
    />,
  );
}

function lensButton(name: string) {
  const lenses = within(screen.getByRole("list", { name: "Lenses" }));
  return lenses.getByRole("button", { name: new RegExp(`^${name}`, "u") });
}

function channelRows() {
  const channels = within(screen.getByRole("list", { name: "Channels" }));
  return channels.getAllByRole("listitem");
}

test("the sidebar lists the lenses, with the chosen one as the current page", () => {
  renderSidebar([], null, "tasks");

  expect(lensButton("Flow").getAttribute("aria-current")).toBe("false");
  expect(lensButton("Tasks").getAttribute("aria-current")).toBe("page");
});

test("each lens says the key that chooses it", () => {
  renderSidebar([]);

  expect(lensButton("Flow").getAttribute("aria-keyshortcuts")).toBe("2");
  expect(lensButton("Tasks").getAttribute("aria-keyshortcuts")).toBe("3");
});

test("the sidebar lists the board's channels as a tree", () => {
  renderSidebar(["fleet/api", "fleet/www", "general"]);

  const rows = channelRows().map((row) => ({
    name: row.textContent,
    depth: row.getAttribute("data-depth"),
  }));

  expect(rows).toEqual([
    { name: "fleet", depth: "0" },
    { name: "#api", depth: "1" },
    { name: "#www", depth: "1" },
    { name: "#general", depth: "0" },
  ]);
});

test("a channel that also holds channels is listed once, as a channel", () => {
  renderSidebar(["fleet", "fleet/www"]);

  expect(channelRows().map((row) => row.textContent)).toEqual([
    "#fleet",
    "#www",
  ]);
});

test("the sidebar follows the board's channels as they appear", () => {
  const { rerender } = renderSidebar(["general"]);

  rerender(
    <Sidebar
      channels={["fleet/docs", "general"]}
      chosenChannel={null}
      chosenLens="flow"
      onChooseChannel={nothing}
      onChooseLens={nothing}
    />,
  );

  expect(screen.getByText("#docs")).toBeDefined();
});

test("channels are listed in natural order, so area2 comes before area10", () => {
  renderSidebar(["fleet/area1", "fleet/area10", "fleet/area2"]);

  expect(channelRows().map((row) => row.textContent)).toEqual([
    "fleet",
    "#area1",
    "#area2",
    "#area10",
  ]);
});

test("choosing a channel, or a folder of channels, asks the lens to narrow to it", () => {
  const onChooseChannel = vi.fn();
  render(
    <Sidebar
      channels={["fleet/www", "general"]}
      chosenChannel={null}
      chosenLens="flow"
      onChooseChannel={onChooseChannel}
      onChooseLens={nothing}
    />,
  );

  screen.getByRole("button", { name: "#www" }).click();
  screen.getByRole("button", { name: "fleet" }).click();

  expect(onChooseChannel.mock.calls).toEqual([["fleet/www"], ["fleet"]]);
});

test("the chosen channel is marked, and the chosen lens stays the current page", () => {
  renderSidebar(["fleet/www", "general"], "fleet/www");

  expect(
    screen.getByRole("button", { name: "#www" }).getAttribute("aria-current"),
  ).toBe("true");
  expect(lensButton("Flow").getAttribute("aria-current")).toBe("page");
});

test("choosing a lens, by a click or by its key, asks to show it", () => {
  const onChooseLens = vi.fn();
  render(
    <Sidebar
      channels={["general"]}
      chosenChannel="general"
      chosenLens="flow"
      onChooseChannel={nothing}
      onChooseLens={onChooseLens}
    />,
  );

  lensButton("Tasks").click();
  act(() => {
    fireEvent.keyDown(document.body, { key: "2" });
  });

  expect(onChooseLens.mock.calls).toEqual([["tasks"], ["flow"]]);
});

test("a lens's key typed into a box, or pressed with a modifier, chooses nothing", () => {
  const onChooseLens = vi.fn();
  render(
    <>
      <input aria-label="a box" />
      <Sidebar
        channels={[]}
        chosenChannel={null}
        chosenLens="flow"
        onChooseChannel={nothing}
        onChooseLens={onChooseLens}
      />
    </>,
  );

  act(() => {
    fireEvent.keyDown(screen.getByRole("textbox", { name: "a box" }), {
      key: "3",
    });
    fireEvent.keyDown(document.body, { key: "3", metaKey: true });
  });

  expect(onChooseLens).not.toHaveBeenCalled();
});
