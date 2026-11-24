import { useEffect, useLayoutEffect, useRef } from "react";
import type { Command } from "../commands";
import { dictionary } from "../dictionary";

type SheetElement = React.RefObject<HTMLDivElement | null>;

type Entry = {
  keys: string;
  what: string;
};

const KEYS_THAT_CLOSE_THE_SHEET = new Set(["Escape", "?"]);

function useKeyboardInTheSheet(sheet: SheetElement) {
  useLayoutEffect(() => {
    const focusedBefore = document.activeElement;
    sheet.current?.focus();
    return () => {
      if (focusedBefore instanceof HTMLElement && focusedBefore.isConnected) {
        focusedBefore.focus();
      }
    };
  }, [sheet]);
}

function useKeysOfTheSheet(sheet: SheetElement, onClose: () => void) {
  useEffect(() => {
    const element = sheet.current;
    if (element === null) {
      return;
    }
    const takeTheKey = (event: KeyboardEvent) => {
      event.stopPropagation();
      if (KEYS_THAT_CLOSE_THE_SHEET.has(event.key)) {
        event.preventDefault();
        onClose();
      }
    };
    element.addEventListener("keydown", takeTheKey);
    return () => {
      element.removeEventListener("keydown", takeTheKey);
    };
  }, [sheet, onClose]);
}

function useClosingOnClicksBeside(scrim: SheetElement, onClose: () => void) {
  useEffect(() => {
    const element = scrim.current;
    if (element === null) {
      return;
    }
    const closeOnClicksBeside = (event: MouseEvent) => {
      if (event.target === element) {
        onClose();
      }
    };
    element.addEventListener("click", closeOnClicksBeside);
    return () => {
      element.removeEventListener("click", closeOnClicksBeside);
    };
  }, [scrim, onClose]);
}

function Group({ title, entries }: { title: string; entries: Entry[] }) {
  return (
    <section className="px-4 py-3">
      <h3 className="mb-2 font-mono text-[10.5px] text-ink-3 uppercase tracking-[0.06em]">
        {title}
      </h3>
      <ul className="flex flex-col gap-1.5">
        {entries.map((entry) => (
          <li
            key={entry.what}
            className="flex items-center justify-between gap-4 text-[12.5px] text-ink-2"
          >
            <span>{entry.what}</span>
            <kbd className="rounded-[3px] border border-line bg-raised px-1.5 py-0.5 font-mono text-[10.5px] text-ink-3 leading-none">
              {entry.keys}
            </kbd>
          </li>
        ))}
      </ul>
    </section>
  );
}

export function ShortcutSheet({
  commands,
  onClose,
}: {
  commands: Command[];
  onClose: () => void;
}) {
  const scrim = useRef<HTMLDivElement>(null);
  const sheet = useRef<HTMLDivElement>(null);
  useKeyboardInTheSheet(sheet);
  useKeysOfTheSheet(sheet, onClose);
  useClosingOnClicksBeside(scrim, onClose);
  const commandEntries = commands.map((command) => ({
    keys: command.shortcut,
    what: command.title,
  }));
  return (
    <div
      ref={scrim}
      className="fixed inset-0 z-20 flex items-start justify-center bg-scrim px-4 pt-[70px]"
    >
      <div
        ref={sheet}
        role="dialog"
        aria-modal={true}
        aria-label={dictionary.shortcuts.title}
        tabIndex={-1}
        className="w-[min(460px,100%)] overflow-hidden rounded-[10px] border border-line-strong bg-raised shadow-[0_24px_60px_var(--shadow)] outline-none"
      >
        <h2 className="border-line border-b px-4 py-3 font-medium text-[13px] text-ink">
          {dictionary.shortcuts.title}
        </h2>
        <Group title={dictionary.shortcuts.commands} entries={commandEntries} />
        <Group
          title={dictionary.shortcuts.moving}
          entries={dictionary.shortcuts.keys}
        />
      </div>
    </div>
  );
}
