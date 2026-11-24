import { cleanup, fireEvent, renderHook } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import { commandsOf } from "../commands";
import type { Choice } from "../frame/choice";
import { useShortcuts } from "../shortcuts";

afterEach(() => {
  cleanup();
  document.body.replaceChildren();
});

function actions() {
  return { showLens: vi.fn(), widen: vi.fn(), showShortcuts: vi.fn() };
}

function shortcutsWhile(choice: Choice) {
  const spies = actions();
  renderHook(() => useShortcuts(commandsOf(choice, spies)));
  return spies;
}

function press(key: string, target: Element = document.body, shiftKey = false) {
  return fireEvent.keyDown(target, { key, shiftKey });
}

test("a lens's key goes to that lens", () => {
  const spies = shortcutsWhile({ lens: "flow", channel: null });

  const leftToThePage = press("3");

  expect(spies.showLens).toHaveBeenCalledWith("tasks");
  expect(leftToThePage).toBe(false);
});

test("narrowed to a channel, the key of the lens on screen shows the whole board", () => {
  const spies = shortcutsWhile({ lens: "flow", channel: "fleet/www" });

  press("2");

  expect(spies.widen).toHaveBeenCalledOnce();
  expect(spies.showLens).not.toHaveBeenCalled();
});

test("? shows the sheet of shortcuts, though it takes shift to type", () => {
  const spies = shortcutsWhile({ lens: "flow", channel: null });

  press("?", document.body, true);

  expect(spies.showShortcuts).toHaveBeenCalledOnce();
});

test("a key typed into a box, or pressed with command, control or option, runs nothing", () => {
  const spies = shortcutsWhile({ lens: "flow", channel: null });
  const box = document.createElement("input");
  document.body.append(box);

  press("3", box);
  fireEvent.keyDown(document.body, { key: "3", metaKey: true });
  fireEvent.keyDown(document.body, { key: "3", ctrlKey: true });
  fireEvent.keyDown(document.body, { key: "3", altKey: true });

  expect(spies.showLens).not.toHaveBeenCalled();
});

test("a key no command has runs nothing and is left to the page", () => {
  const spies = shortcutsWhile({ lens: "flow", channel: null });

  const leftToThePage = press("x");

  expect(leftToThePage).toBe(true);
  expect(spies.showLens).not.toHaveBeenCalled();
  expect(spies.showShortcuts).not.toHaveBeenCalled();
});

test("a key another part of the app has taken runs nothing", () => {
  const spies = shortcutsWhile({ lens: "flow", channel: null });
  const takenByTheList = (event: KeyboardEvent) => event.preventDefault();
  document.body.addEventListener("keydown", takenByTheList);

  press("3");

  document.body.removeEventListener("keydown", takenByTheList);
  expect(spies.showLens).not.toHaveBeenCalled();
});
