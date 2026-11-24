import { useEffect } from "react";
import type { Command } from "./commands";

function isTypingInto(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }
  const takesText =
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement;
  return takesText || target.isContentEditable;
}

function isPressedWithCommandControlOrOption(event: KeyboardEvent): boolean {
  return event.metaKey || event.ctrlKey || event.altKey;
}

function commandTakingEachShortcut(commands: Command[]): Map<string, Command> {
  return new Map(commands.map((command) => [command.shortcut, command]));
}

export function useShortcuts(commands: Command[]) {
  useEffect(() => {
    const commandOfTheShortcut = commandTakingEachShortcut(commands);
    const runTheCommandOfTheKey = (event: KeyboardEvent) => {
      const leftAlone =
        event.defaultPrevented ||
        isPressedWithCommandControlOrOption(event) ||
        isTypingInto(event.target);
      if (leftAlone) {
        return;
      }
      const command = commandOfTheShortcut.get(event.key);
      if (command === undefined) {
        return;
      }
      event.preventDefault();
      command.run();
    };
    document.addEventListener("keydown", runTheCommandOfTheKey);
    return () => {
      document.removeEventListener("keydown", runTheCommandOfTheKey);
    };
  }, [commands]);
}
