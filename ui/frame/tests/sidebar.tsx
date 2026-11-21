import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import { Sidebar } from "../sidebar";

afterEach(cleanup);

const nothing = () => undefined;

function renderSidebar(
  channels: string[],
  chosenChannel: string | null = null,
) {
  return render(
    <Sidebar
      channels={channels}
      chosenChannel={chosenChannel}
      onChooseChannel={nothing}
      onChooseFlow={nothing}
    />,
  );
}

function channelRows() {
  const channels = within(screen.getByRole("list", { name: "Channels" }));
  return channels.getAllByRole("listitem");
}

test("the sidebar lists the Flow lens, chosen", () => {
  renderSidebar([]);

  const lenses = within(screen.getByRole("list", { name: "Lenses" }));

  expect(
    lenses
      .getByText("Flow")
      .closest("[aria-current]")
      ?.getAttribute("aria-current"),
  ).toBe("page");
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
      onChooseChannel={nothing}
      onChooseFlow={nothing}
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

test("choosing a channel, or a folder of channels, asks Flow to narrow to it", () => {
  const onChooseChannel = vi.fn();
  render(
    <Sidebar
      channels={["fleet/www", "general"]}
      chosenChannel={null}
      onChooseChannel={onChooseChannel}
      onChooseFlow={nothing}
    />,
  );

  screen.getByRole("button", { name: "#www" }).click();
  screen.getByRole("button", { name: "fleet" }).click();

  expect(onChooseChannel.mock.calls).toEqual([["fleet/www"], ["fleet"]]);
});

test("the chosen channel is marked, and the Flow lens is no longer the current page", () => {
  renderSidebar(["fleet/www", "general"], "fleet/www");

  expect(
    screen.getByRole("button", { name: "#www" }).getAttribute("aria-current"),
  ).toBe("true");
  expect(
    screen.getByRole("button", { name: "Flow" }).getAttribute("aria-current"),
  ).toBe("false");
});

test("choosing Flow widens it to the whole board", () => {
  const onChooseFlow = vi.fn();
  render(
    <Sidebar
      channels={["general"]}
      chosenChannel="general"
      onChooseChannel={nothing}
      onChooseFlow={onChooseFlow}
    />,
  );

  screen.getByRole("button", { name: "Flow" }).click();

  expect(onChooseFlow).toHaveBeenCalledOnce();
});
