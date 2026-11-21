import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { Sidebar } from "../sidebar";

afterEach(cleanup);

function channelRows() {
  const channels = within(screen.getByRole("list", { name: "Channels" }));
  return channels.getAllByRole("listitem");
}

test("the sidebar lists the Flow lens, chosen", () => {
  render(<Sidebar channels={[]} />);

  const lenses = within(screen.getByRole("list", { name: "Lenses" }));

  expect(
    lenses
      .getByText("Flow")
      .closest("[aria-current]")
      ?.getAttribute("aria-current"),
  ).toBe("page");
});

test("the sidebar lists the board's channels as a tree", () => {
  render(<Sidebar channels={["fleet/api", "fleet/www", "general"]} />);

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
  render(<Sidebar channels={["fleet", "fleet/www"]} />);

  expect(channelRows().map((row) => row.textContent)).toEqual([
    "#fleet",
    "#www",
  ]);
});

test("the sidebar follows the board's channels as they appear", () => {
  const { rerender } = render(<Sidebar channels={["general"]} />);

  rerender(<Sidebar channels={["fleet/docs", "general"]} />);

  expect(screen.getByText("#docs")).toBeDefined();
});

test("channels are listed in natural order, so area2 comes before area10", () => {
  render(<Sidebar channels={["fleet/area1", "fleet/area10", "fleet/area2"]} />);

  expect(channelRows().map((row) => row.textContent)).toEqual([
    "fleet",
    "#area1",
    "#area2",
    "#area10",
  ]);
});
