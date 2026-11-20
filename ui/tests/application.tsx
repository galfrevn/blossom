import { act, cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { Application } from "../application";
import type { OpenBoard } from "../core/contract";
import { createFakeCore } from "../core/fake";
import { createOpenBoardStore } from "../core/store";

afterEach(cleanup);

function renderApplicationWhileTheCoreSays(openBoard?: OpenBoard) {
  const fakeCore = createFakeCore();
  const openBoardStore = createOpenBoardStore(fakeCore.core);
  if (openBoard !== undefined) {
    fakeCore.push(openBoard);
  }
  render(<Application openBoardStore={openBoardStore} />);
  return fakeCore;
}

test("the window is dragged by its title bar", () => {
  renderApplicationWhileTheCoreSays({ kind: "noneRunning" });

  const titleBar = screen.getByRole("banner");

  expect(titleBar.hasAttribute("data-tauri-drag-region")).toBe(true);
});

test("with no board running, it shows how to start one", () => {
  renderApplicationWhileTheCoreSays({ kind: "noneRunning" });

  expect(
    screen.getByRole("heading", { name: "No board running" }),
  ).toBeDefined();
  expect(screen.getByText("vb serve")).toBeDefined();
});

test("with a board open, the status bar shows its address and who you are on it", () => {
  renderApplicationWhileTheCoreSays({
    kind: "open",
    url: "http://127.0.0.1:4100/api/board",
    person: { handle: "ada", name: "Ada Lovelace" },
  });

  const statusBar = screen.getByRole("contentinfo");

  expect(
    within(statusBar).getByText("http://127.0.0.1:4100/api/board"),
  ).toBeDefined();
  expect(within(statusBar).getByText("Ada Lovelace")).toBeDefined();
  expect(within(statusBar).getByText("@ada")).toBeDefined();
  expect(
    screen.queryByRole("heading", { name: "No board running" }),
  ).toBeNull();
});

test("until the core answers, it shows neither screen", () => {
  renderApplicationWhileTheCoreSays();

  expect(
    screen.queryByRole("heading", { name: "No board running" }),
  ).toBeNull();
  expect(screen.queryByRole("contentinfo")).toBeNull();
});

test("the screen follows the core as a board starts and stops", () => {
  const fakeCore = renderApplicationWhileTheCoreSays({ kind: "noneRunning" });

  act(() => {
    fakeCore.push({
      kind: "open",
      url: "http://127.0.0.1:4100/api/board",
      person: { handle: "ada", name: "Ada Lovelace" },
    });
  });

  expect(
    within(screen.getByRole("contentinfo")).getByText("Ada Lovelace"),
  ).toBeDefined();
  expect(
    screen.queryByRole("heading", { name: "No board running" }),
  ).toBeNull();

  act(() => {
    fakeCore.push({ kind: "noneRunning" });
  });

  expect(
    screen.getByRole("heading", { name: "No board running" }),
  ).toBeDefined();
  expect(screen.queryByRole("contentinfo")).toBeNull();
});
