import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import { commandsOf } from "../../commands";
import { dictionary } from "../../dictionary";
import { ShortcutSheet } from "../sheet";

afterEach(cleanup);

function noActions() {
  return { showLens: vi.fn(), widen: vi.fn(), showShortcuts: vi.fn() };
}

function renderTheSheet(channel: string | null = null) {
  const onClose = vi.fn();
  render(
    <ShortcutSheet
      commands={commandsOf({ lens: "flow", channel }, noActions())}
      onClose={onClose}
    />,
  );
  return { onClose, sheet: screen.getByRole("dialog") };
}

function keyBeside(title: string): string | null {
  const entry = screen
    .getAllByRole("listitem")
    .find((item) => within(item).queryByText(title) !== null);
  return entry?.querySelector("kbd")?.textContent ?? null;
}

test("the sheet lists every command with its shortcut", () => {
  renderTheSheet("fleet/www");

  expect(keyBeside(dictionary.commands.goTo("Flow"))).toBe("2");
  expect(keyBeside(dictionary.commands.goTo("Tasks"))).toBe("3");
  expect(keyBeside(dictionary.lenses.widen)).toBe("2");
  expect(keyBeside(dictionary.commands.showShortcuts)).toBe("?");
});

test("the sheet lists the keys that move through a list and the finder's", () => {
  renderTheSheet();

  for (const { keys, what } of dictionary.shortcuts.keys) {
    expect(keyBeside(what)).toBe(keys);
  }
});

test("the sheet is named, and takes the keyboard while it is open", () => {
  const { sheet } = renderTheSheet();

  expect(sheet.getAttribute("aria-label")).toBe(dictionary.shortcuts.title);
  expect(sheet.contains(document.activeElement)).toBe(true);
});

test("escape or ? closes the sheet, and neither reaches the app behind it", () => {
  const { onClose, sheet } = renderTheSheet();
  const keysBehind = vi.fn();
  document.addEventListener("keydown", keysBehind);

  act(() => {
    fireEvent.keyDown(sheet, { key: "Escape" });
    fireEvent.keyDown(sheet, { key: "?", shiftKey: true });
    fireEvent.keyDown(sheet, { key: "3" });
  });

  document.removeEventListener("keydown", keysBehind);
  expect(onClose).toHaveBeenCalledTimes(2);
  expect(keysBehind).not.toHaveBeenCalled();
});

test("a click beside the sheet closes it, and a click on it does not", () => {
  const { onClose, sheet } = renderTheSheet();

  act(() => {
    sheet.click();
  });
  expect(onClose).not.toHaveBeenCalled();

  act(() => {
    sheet.parentElement?.click();
  });
  expect(onClose).toHaveBeenCalledOnce();
});
