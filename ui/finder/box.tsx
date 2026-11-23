import { useCallback, useEffect, useState } from "react";
import type { Command } from "../commands";
import type { SearchResults } from "../core/contract";
import { dictionary } from "../dictionary";
import { FinderDialog } from "./dialog";

function isCommandK(event: KeyboardEvent): boolean {
  const otherModifiers = event.ctrlKey || event.altKey || event.shiftKey;
  return event.metaKey && !otherModifiers && event.key.toLowerCase() === "k";
}

function useCommandK(toggle: () => void) {
  useEffect(() => {
    const toggleOnCommandK = (event: KeyboardEvent) => {
      if (isCommandK(event)) {
        event.preventDefault();
        toggle();
      }
    };
    document.addEventListener("keydown", toggleOnCommandK);
    return () => {
      document.removeEventListener("keydown", toggleOnCommandK);
    };
  }, [toggle]);
}

function OpenFinder({
  isOpen,
  search,
  onOpenThread,
  onNarrowTo,
  onClose,
  commands,
}: {
  isOpen: boolean;
  search: (query: string) => Promise<SearchResults>;
  onOpenThread: (messageId: string) => void;
  onNarrowTo: (channel: string) => void;
  onClose: () => void;
  commands: Command[];
}) {
  if (!isOpen) {
    return null;
  }
  return (
    <FinderDialog
      search={search}
      onOpenThread={onOpenThread}
      onNarrowTo={onNarrowTo}
      onClose={onClose}
      commands={commands}
    />
  );
}

export function FinderBox({
  search,
  onOpenThread,
  onNarrowTo,
  commands,
}: {
  search: (query: string) => Promise<SearchResults>;
  onOpenThread: (messageId: string) => void;
  onNarrowTo: (channel: string) => void;
  commands: Command[];
}) {
  const [isOpen, setIsOpen] = useState(false);
  const open = useCallback(() => setIsOpen(true), []);
  const close = useCallback(() => setIsOpen(false), []);
  const toggle = useCallback(() => setIsOpen((wasOpen) => !wasOpen), []);
  useCommandK(toggle);
  return (
    <>
      <button
        type="button"
        aria-label={dictionary.finder.box}
        aria-keyshortcuts="Meta+K"
        onClick={open}
        className="inline-flex min-w-[240px] items-center gap-2.5 rounded-[6px] border border-line bg-raised py-1 pr-1.5 pl-2.5 text-[12px] text-ink-3 hover:border-line-strong hover:text-ink-2"
      >
        <span className="flex-1 text-left">{dictionary.finder.box}</span>
        <kbd className="rounded-[3px] border border-line bg-raised px-1 py-0.5 font-mono text-[10px] text-ink-3 leading-none">
          {dictionary.finder.boxKey}
        </kbd>
      </button>
      <OpenFinder
        isOpen={isOpen}
        search={search}
        onOpenThread={onOpenThread}
        onNarrowTo={onNarrowTo}
        onClose={close}
        commands={commands}
      />
    </>
  );
}
